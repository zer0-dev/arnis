use crate::cartesian::XZPoint;

pub struct Ground {
    //
}

impl Ground {
    pub fn new() -> Self {
        Self {}
    }

    #[inline(always)]
    pub fn level(&self, coord: XZPoint) -> i32 {
        (20.0 * (coord.x as f64 / 100.0).sin() + 20.0 * (coord.z as f64 / 100.0).sin()) as i32
    }

    #[inline(always)]
    pub fn min_level<I: Iterator<Item = XZPoint>>(&self, coords: I) -> Option<i32> {
        coords.map(|c| self.level(c)).min()
    }

    #[inline(always)]
    pub fn max_level<I: Iterator<Item = XZPoint>>(&self, coords: I) -> Option<i32> {
        coords.map(|c| self.level(c)).max()
    }
}
