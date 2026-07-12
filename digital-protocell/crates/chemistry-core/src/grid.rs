//! Grid geometry and petri-dish mask.

use crate::config::{DISH_RADIUS, GRID_HEIGHT, GRID_WIDTH, RESERVOIR_WIDTH};

#[derive(Debug, Clone)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub cx: f64,
    pub cy: f64,
    pub dish_mask: Vec<bool>,
    pub reservoir_mask: Vec<bool>,
    pub no_flux_mask: Vec<bool>,
}

impl Grid {
    pub fn new() -> Self {
        let width = GRID_WIDTH;
        let height = GRID_HEIGHT;
        let cx = (width as f64 - 1.0) / 2.0;
        let cy = (height as f64 - 1.0) / 2.0;
        let n = width * height;
        let mut dish_mask = vec![false; n];
        let mut reservoir_mask = vec![false; n];
        let mut no_flux_mask = vec![false; n];

        for j in 0..height {
            for i in 0..width {
                let idx = Self::index(width, i, j);
                let dx = i as f64 - cx;
                let dy = j as f64 - cy;
                let r = (dx * dx + dy * dy).sqrt();
                if r <= DISH_RADIUS {
                    dish_mask[idx] = true;
                    no_flux_mask[idx] = true;
                    if r > DISH_RADIUS - RESERVOIR_WIDTH {
                        reservoir_mask[idx] = true;
                    }
                }
            }
        }

        Self {
            width,
            height,
            cx,
            cy,
            dish_mask,
            reservoir_mask,
            no_flux_mask,
        }
    }

    #[inline]
    pub fn index(width: usize, i: usize, j: usize) -> usize {
        j * width + i
    }

    #[inline]
    pub fn in_dish(&self, idx: usize) -> bool {
        self.dish_mask[idx]
    }

    pub fn distance_from_center(&self, i: usize, j: usize) -> f64 {
        let dx = i as f64 - self.cx;
        let dy = j as f64 - self.cy;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn dish_cell_count(&self) -> usize {
        self.dish_mask.iter().filter(|&&m| m).count()
    }
}

impl Default for Grid {
    fn default() -> Self {
        Self::new()
    }
}
