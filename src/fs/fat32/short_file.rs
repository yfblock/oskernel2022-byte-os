use alloc::string::{String, ToString};

use super::{FAT32FileItemAttr, file_trait::FilesystemItemOperator};

// FAT32短文件目录项
#[allow(dead_code)]
#[repr(packed)]
pub struct FAT32shortFileItem {
    pub filename: [u8; 8],          // 文件名
    pub ext: [u8; 3],               // 扩展名
    pub attr: FAT32FileItemAttr,    // 属性
    pub system_reserved: u8,        // 系统保留
    pub create_time_10ms: u8,       // 创建时间的10毫秒位
    pub create_time: u16,           // 创建时间
    pub create_date: u16,           // 创建日期
    pub last_access_date: u16,      // 最后访问日期
    pub start_high: u16,            // 起始簇号的高16位
    pub last_modify_time: u16,      // 最近修改时间
    pub last_modify_date: u16,      // 最近修改日期
    pub start_low: u16,             // 起始簇号的低16位
    pub len: u32                    // 文件长度
}

impl FilesystemItemOperator for FAT32shortFileItem {
    // 获取文件名
    fn filename(&self) -> String {
        let filename = String::from_utf8_lossy(&self.filename);
        // 获取文件名总长度
        let mut filename_size = filename.len();
        // 获取有效文件名长度
        for i in filename.chars().rev() {
            if !i.is_whitespace() { break; }
            filename_size = filename_size - 1;
        }
        // 拼接得到文件名
        let filename = filename[..filename_size].to_string();
        let ext = String::from_utf8_lossy(&self.ext);
        if ext.trim() == "" {
            filename 
        } else {
            filename + "." + &ext
        }
    }

    // 获取文件大小
    fn file_size(&self) -> usize {
        self.len as usize
    }

    // 获取文件开始簇
    fn start_cluster(&self) -> usize {
        (self.start_high as usize) << 16 | self.start_low as usize
    }
    // 获取文件属性
    fn get_attr(&self) -> FAT32FileItemAttr {
        self.attr.clone()
    }
}
