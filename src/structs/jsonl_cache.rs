use anyhow::{Context, Result};
use memmap2::Mmap;
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};
use std::marker::PhantomData;
use std::path::PathBuf;

#[derive(Debug)]
pub struct JsonCache<T> {
    path: PathBuf,
    _marker: PhantomData<T>,
}

impl<T> JsonCache<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    pub fn new(path: PathBuf) -> Self {
        JsonCache {
            path,
            _marker: PhantomData,
        }
    }

    pub fn write(&self, obj: T) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;

        let json = serde_json::to_string(&obj)?;
        writeln!(file, "{}", json)?;
        Ok(())
    }

    pub fn get_all_objects(&self) -> Result<Vec<T>> {
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        reader
            .lines()
            .map(|line| serde_json::from_str(&line.unwrap()).context("cannot parse line"))
            .collect()
    }

    pub fn get_first_objects(&self, count: usize) -> Result<Vec<T>> {
        let file = File::open(&self.path)?;
        let mut reader = BufReader::new(file);
        let mut objects = Vec::new();
        let mut line = String::new();

        for _ in 0..count {
            if reader.read_line(&mut line)? == 0 {
                break;
            }
            if let Ok(obj) = serde_json::from_str(&line.trim()) {
                objects.push(obj);
            }
            line.clear();
        }

        Ok(objects)
    }

    // Function to count the number of objects in the cache
    pub fn count_objects(&self) -> Result<usize> {
        let file = File::open(&self.path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        let lines = mmap.split(|&b| b == b'\n').filter(|&b| b.len() > 0);
        return Ok(lines.count());
    }

    // Function to check if the cache is empty
    pub fn is_empty(&self) -> bool {
        if !fs::exists(&self.path).unwrap() {
            return true;
        }
        let file = File::open(&self.path).unwrap();
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap() == 0
    }

    pub fn get_last_objects(&self, count: usize) -> Result<Vec<T>> {
        let file = File::open(&self.path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        let mut objects = Vec::with_capacity(count);
        let mut lines = mmap.split(|&b| b == b'\n').filter(|&b| b.len() > 0).rev();
        let mut line_buffer = String::new();

        for _ in 0..count {
            if let Some(line) = lines.next() {
                line_buffer.clear();
                line_buffer.push_str(std::str::from_utf8(line).context("invalid UTF-8 sequence")?);
                if let Ok(obj) = serde_json::from_str(&line_buffer.trim()) {
                    objects.push(obj);
                }
            } else {
                break;
            }
        }

        objects.reverse();
        Ok(objects)
    }
}
