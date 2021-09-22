use crate::cfb::Cfb;
use crate::cfb::fat::Fat;
use crate::cfb::fat::sector_number::SectorNumber;
use crate::cfb::header::SectorCount;

macro_rules! populated_fats {
    ($count:expr) => {
        {
            let mut vec = Vec::new();
            vec.reserve_exact($count);
            for _ in 0..$count {
                vec.push(None);
            }
            vec
        }
    };
}

pub(crate) struct Cache<'a> {
    cfb: &'a Cfb,
    fats: Vec<Option<Fat>>,
    mini_fats: Vec<Option<Fat>>,
    no_of_sectors_per_fat: SectorCount,
}

impl<'a> Cache<'a> {
    pub(crate) fn new(cfb: &'a Cfb) -> Self {
        let header = cfb.header();
        let no_of_sectors_per_fat = SectorCount(cfb.sector_size >> std::mem::size_of::<SectorNumber>());
        let fats = populated_fats!(109);
        let mini_fats = populated_fats!(header.no_of_fat_sectors().0 as usize);

        Self {
            cfb,
            fats,
            mini_fats,
            no_of_sectors_per_fat,
        }
    }

    pub(crate) fn fat(&mut self, sector_no: SectorNumber) -> &Fat {
        let fat_idx = sector_no / self.no_of_sectors_per_fat;

        let ptr = unsafe { self.fats.get_unchecked_mut(fat_idx.0 as usize) };
        if ptr.is_none() {
            let fat = self.cfb.fat(self.cfb.header().sector_no_of_fat(fat_idx));
            *ptr = Some(fat);
        }

        ptr.as_ref().unwrap()
    }

    pub(crate) fn mini_fat(&mut self, sector_no: SectorNumber) -> &Fat {
        let no_of_sectors_per_fat = self.no_of_sectors_per_fat;
        let mini_fat_idx = sector_no / no_of_sectors_per_fat;

        if self.mini_fats[mini_fat_idx.0 as usize].as_ref().is_some() {
            self.mini_fats[mini_fat_idx.0 as usize].as_ref().unwrap()
        }
        else {
            let mut mini_fat_sector_number = self.cfb
                .header()
                .first_mini_fat_sector_location();

            for _ in 0..mini_fat_idx.0 {
                let fat = self.fat(mini_fat_sector_number);
                mini_fat_sector_number = fat
                    .sector_number((mini_fat_sector_number % no_of_sectors_per_fat).0);
            }

            let mini_fat_data = self.cfb.sector_bytes(mini_fat_sector_number);
            let mini_fat = Fat::new(mini_fat_data);

            let ptr = unsafe { self.mini_fats.get_unchecked_mut(mini_fat_idx.0 as usize) };
            ptr.replace(mini_fat);
            ptr.as_ref().unwrap()
        }
    }
}
