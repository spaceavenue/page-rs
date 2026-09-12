use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use anyhow::Context;
use memchr::memchr_iter;
use memmap2::{Mmap, MmapOptions};

use crate::cli::Input;

/// Files below ~64KiB are faster to load using `read()` instead of `mmap()`-ing them.
const MMAP_THRESHOLD: usize = 64 * 1024; // 64 KiB

pub enum BufferSource {
  Mmap(Mmap),
  Heap(Vec<u8>),
}

impl BufferSource {
  #[inline(always)]
  pub fn as_slice(&self) -> &[u8] {
    match self {
      Self::Mmap(m) => m.as_ref(),
      Self::Heap(v) => v.as_slice(),
    }
  }
}

pub struct Buffer {
  source: BufferSource,
  line_offsets: Vec<usize>,
  filename: String,
  size: usize,
}

impl Buffer {
  pub fn load(input: &Input) -> anyhow::Result<Self> {
    match input {
      Input::Path(path) => {
        Self::from_file(path).with_context(|| format!("failed to read file: {}", path.display()))
      }
      Input::Stdin => Self::from_stdin(),
    }
  }

  /// Load a file via Mmap.
  pub fn from_file(path: &Path) -> anyhow::Result<Self> {
    let mut file = File::open(path)?;
    let size = usize::try_from(file.metadata()?.len())
      .context("file is too large to fit in memory address space.")?;

    let source = if size < MMAP_THRESHOLD {
      let mut buf = Vec::with_capacity(size);
      file.read_to_end(&mut buf)?;
      BufferSource::Heap(buf)
    } else {
      let mmap = unsafe { MmapOptions::new().len(size).map(&file)? };
      // hint the kernel to perform bulk readahead and sequential TLB prep
      let _ = mmap.advise(memmap2::Advice::Sequential);
      BufferSource::Mmap(mmap)
    };

    let line_offsets = Self::index_lines(source.as_slice());
    let filename = path
      .file_name()
      .and_then(|f| f.to_str())
      .unwrap_or("file")
      .to_string();

    Ok(Self {
      source,
      line_offsets,
      filename,
      size,
    })
  }

  /// Load bytes from stdin.
  pub fn from_stdin() -> anyhow::Result<Self> {
    let mut stdin = io::stdin().lock();
    let mut data = Vec::with_capacity(64 * 1024);
    stdin.read_to_end(&mut data)?;
    drop(stdin);

    let size = data.len();
    let source = BufferSource::Heap(data);
    let line_offsets = Self::index_lines(source.as_slice());

    Ok(Self {
      source,
      line_offsets,
      filename: "[stdin]".to_string(),
      size,
    })
  }

  /// Newline indexing.
  fn index_lines(bytes: &[u8]) -> Vec<usize> {
    if bytes.is_empty() {
      return vec![0];
    }

    // Heuristic: ~40 bytes per line average
    let mut offsets = Vec::with_capacity(bytes.len() / 40 + 1);
    offsets.push(0);

    memchr_iter(b'\n', bytes)
      .filter(|p| *p + 1 < bytes.len())
      .for_each(|p| offsets.push(p + 1));

    offsets
  }
}
