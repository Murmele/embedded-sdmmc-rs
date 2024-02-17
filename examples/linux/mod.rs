//! Helpers for using embedded-sdmmc on Linux

use async_std::fs::OpenOptions;
use async_std::io::prelude::*;
use async_std::io::SeekFrom;
use async_std::path::Path;
use chrono::Timelike;
use embedded_sdmmc::{Block, BlockCount, BlockDevice, BlockIdx, TimeSource, Timestamp};

#[derive(Debug)]
pub struct LinuxBlockDevice<P: AsRef<Path> + Clone + std::marker::Send + std::marker::Sync> {
    print_blocks: bool,
    device_name: P,
}

impl<P: AsRef<Path> + Clone + std::marker::Send + std::marker::Sync> LinuxBlockDevice<P> {
    pub async fn new(device_name: P, print_blocks: bool) -> Result<Self, std::io::Error>
    where
        P: AsRef<Path>,
    {
        Ok(LinuxBlockDevice {
            print_blocks,
            device_name: device_name,
        })
    }
}

impl<P: AsRef<Path> + Clone + std::marker::Send + std::marker::Sync> BlockDevice
    for LinuxBlockDevice<P>
{
    type Error = std::io::Error;

    async fn read(
        &self,
        blocks: &mut [Block],
        start_block_idx: BlockIdx,
        reason: &str,
    ) -> Result<(), Self::Error> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(self.device_name.clone())
            .await?;
        file.seek(SeekFrom::Start(start_block_idx.into_bytes()))
            .await?;
        for block in blocks.iter_mut() {
            file.read_exact(&mut block.contents).await?;
            if self.print_blocks {
                println!(
                    "Read block ({}) {:?}: {:?}",
                    reason, start_block_idx, &block
                );
            }
        }
        Ok(())
    }

    async fn write(&self, blocks: &[Block], start_block_idx: BlockIdx) -> Result<(), Self::Error> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(self.device_name.clone())
            .await?;
        file.seek(SeekFrom::Start(start_block_idx.into_bytes()))
            .await?;
        for block in blocks.iter() {
            file.write_all(&block.contents).await?;
            if self.print_blocks {
                println!("Wrote: {:?}", &block);
            }
        }
        Ok(())
    }

    async fn num_blocks(&self) -> Result<BlockCount, Self::Error> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(self.device_name.clone())
            .await?;
        let num_blocks = file.metadata().await.unwrap().len() / 512;
        Ok(BlockCount(num_blocks as u32))
    }
}

#[derive(Debug)]
pub struct Clock;

impl TimeSource for Clock {
    fn get_timestamp(&self) -> Timestamp {
        use chrono::Datelike;
        let local: chrono::DateTime<chrono::Local> = chrono::Local::now();
        Timestamp {
            year_since_1970: (local.year() - 1970) as u8,
            zero_indexed_month: local.month0() as u8,
            zero_indexed_day: local.day0() as u8,
            hours: local.hour() as u8,
            minutes: local.minute() as u8,
            seconds: local.second() as u8,
        }
    }
}

// ****************************************************************************
//
// End Of File
//
// ****************************************************************************
