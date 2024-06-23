//! File opening related tests

use std::borrow::Borrow;

use embedded_sdmmc::{Mode, VolumeIdx, VolumeManager, VolumeOpenMode};

mod utils;

#[test]
fn append_file() {
    let time_source = utils::make_time_source();
    let disk = utils::make_block_device(utils::DISK_SOURCE).unwrap();
    let mut volume_mgr: VolumeManager<utils::RamDisk<Vec<u8>>, utils::TestTimeSource, 4, 2, 1> =
        VolumeManager::new_with_limits(disk, time_source, 0xAA00_0000);
    let volume = volume_mgr
        .open_raw_volume(VolumeIdx(0), VolumeOpenMode::ReadWrite)
        .expect("open volume");
    let root_dir = volume_mgr.open_root_dir(volume).expect("open root dir");

    // Open with string
    let f = volume_mgr
        .open_file_in_dir(root_dir, "T.TXT", Mode::ReadWriteCreateOrTruncate)
        .expect("open file");

    // Should be enough to cause a few more clusters to be allocated
    const NUMBER_ELEMENTS: u32 = 1024 * 1024*10;
    let test_data = vec![0xAB; NUMBER_ELEMENTS as usize];
    volume_mgr.write(f, &test_data).expect("file write");

    let length = volume_mgr.file_length(f).expect("get length");
    assert_eq!(length, NUMBER_ELEMENTS);

    let offset = volume_mgr.file_offset(f).expect("offset");
    assert_eq!(offset, NUMBER_ELEMENTS);

    // Now wind it back 1 byte;
    volume_mgr.file_seek_from_current(f, -1).expect("Seeking");

    let offset = volume_mgr.file_offset(f).expect("offset");
    assert_eq!(offset, (NUMBER_ELEMENTS) - 1);

    // Write another megabyte, making `2 MiB - 1`
    volume_mgr.write(f, &test_data).expect("file write");

    let length = volume_mgr.file_length(f).expect("get length");
    assert_eq!(length, (NUMBER_ELEMENTS * 2) - 1);

    volume_mgr.close_file(f).expect("close dir");

    // Now check the file length again

    let entry = volume_mgr
        .find_directory_entry(root_dir, "T.TXT")
        .expect("Find entry");
    assert_eq!(entry.size, (NUMBER_ELEMENTS * 2) - 1);

    let f = volume_mgr.open_file_in_dir(root_dir, "T.txt", Mode::ReadOnly).unwrap();
    let mut buffer = [0 as u8; 1000];
    volume_mgr.read(f, buffer.as_mut()).unwrap();
    volume_mgr.close_file(f).unwrap();

    volume_mgr.delete_file_in_dir(root_dir, "README.txt").unwrap();

    volume_mgr.close_dir(root_dir).expect("close dir");
    volume_mgr.close_volume(volume).expect("close volume");

    let c = volume_mgr.block_device.content();

    utils::pack_disk(c.borrow());
}

#[test]
fn flush_file() {
    let time_source = utils::make_time_source();
    let disk = utils::make_block_device(utils::DISK_SOURCE).unwrap();
    let mut volume_mgr: VolumeManager<utils::RamDisk<Vec<u8>>, utils::TestTimeSource, 4, 2, 1> =
        VolumeManager::new_with_limits(disk, time_source, 0xAA00_0000);
    let volume = volume_mgr
        .open_raw_volume(VolumeIdx(0), VolumeOpenMode::ReadWrite)
        .expect("open volume");
    let root_dir = volume_mgr.open_root_dir(volume).expect("open root dir");

    const FILE: &str = "RME123.TXT";
    // Open with string
    let f = volume_mgr
        .open_file_in_dir(root_dir, FILE, Mode::ReadWriteCreate)
        .expect("open file");

    // Write some data to the file
    let test_data = vec![0xCC; 64];
    volume_mgr.write(f, &test_data).expect("file write");

    // Check that the file length is zero in the directory entry, as we haven't
    // flushed yet
    let entry = volume_mgr
        .find_directory_entry(root_dir, FILE)
        .expect("find entry");
    assert_eq!(entry.size, 0);

    volume_mgr.flush_file(f).expect("flush");

    // Now check the file length again after flushing
    let entry = volume_mgr
        .find_directory_entry(root_dir, FILE)
        .expect("find entry");
    assert_eq!(entry.size, 64);

    // Flush more writes
    volume_mgr.write(f, &test_data).expect("file write");
    volume_mgr.write(f, &test_data).expect("file write");
    volume_mgr.flush_file(f).expect("flush");

    // Now check the file length again, again
    let entry = volume_mgr
        .find_directory_entry(root_dir, FILE)
        .expect("find entry");
    assert_eq!(entry.size, 64 * 3);

    volume_mgr.close_file(f).unwrap();

    let f = volume_mgr.open_file_in_dir(root_dir, FILE, Mode::ReadOnly).unwrap();
    let mut buffer = [0 as u8; 1000];
    volume_mgr.read(f, buffer.as_mut()).unwrap();

    volume_mgr.close_file(f).unwrap();

    volume_mgr.close_dir(root_dir).unwrap();
    volume_mgr.close_volume(volume).unwrap();

    let c = volume_mgr.block_device.content();

    utils::pack_disk(c.borrow());
}

#[test]
fn write_file_not_correct_closed() {
    let time_source = utils::make_time_source();
    let disk = utils::make_block_device(utils::DISK_SOURCE).unwrap();
    let mut volume_mgr: VolumeManager<utils::RamDisk<Vec<u8>>, utils::TestTimeSource, 4, 2, 1> =
        VolumeManager::new_with_limits(disk, time_source, 0xAA00_0000);
    let volume = volume_mgr
        .open_raw_volume(VolumeIdx(0), VolumeOpenMode::ReadWrite)
        .expect("open volume");
    let root_dir = volume_mgr.open_root_dir(volume).expect("open root dir");

    // Open with string
    let f = volume_mgr
        .open_file_in_dir(root_dir, "README.TXT", Mode::ReadWriteTruncate)
        .expect("open file");

    // Write some data to the file
    volume_mgr.write(f, b"Hello").expect("file write");

    // Check that the file length is zero in the directory entry, as we haven't
    // flushed yet
    let entry = volume_mgr
        .find_directory_entry(root_dir, "README.TXT")
        .expect("find entry");
    assert_eq!(entry.size, 0);

    volume_mgr.flush_file(f).expect("flush");
    volume_mgr.close_file(f).unwrap();
    volume_mgr.close_dir(root_dir).unwrap();
    volume_mgr.close_volume(volume).unwrap();

    let c = volume_mgr.block_device.content();

    utils::pack_disk(c.borrow());

    // let disk = utils::make_block_device_no_zip(c.borrow());
    // let time_source = utils::make_time_source();
    // let mut volume_mgr: VolumeManager<utils::RamDisk<Vec<u8>>, utils::TestTimeSource, 4, 2, 1> =
    //     VolumeManager::new_with_limits(disk, time_source, 0xAA00_0000);
    // let volume = volume_mgr
    //     .open_raw_volume(VolumeIdx(0), VolumeOpenMode::ReadWrite)
    //     .expect("open volume");

    // // Dirty because it was not properly closed before
    // assert_eq!(volume_mgr.volume_status_dirty(volume).unwrap(), true);
}

// ****************************************************************************
//
// End Of File
//
// ****************************************************************************
