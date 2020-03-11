mod utils;

extern crate web_sys;
extern crate rand;

use wasm_bindgen::prelude::*;
use std::fmt;
use rand::Rng;

// TODO: Add ability to slow down

// Macro to simplify logging.
#[allow(unused_macros)]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    };
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
// Primitive representation (https://doc.rust-lang.org/reference/type-layout.html)
// Keeps each Cell to a single byte.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1
}

impl Cell {
    fn toggle(&mut self) {
        *self = match *self {
            Cell::Dead => Cell::Alive,
            Cell::Alive => Cell::Dead,
        };
    }
}


#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>
}

// Methods not being exported to Javascript
impl Universe {
    fn get_index(&self, row: u32, column: u32) ->  usize {
        (row * self.width + column) as usize
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                count += self.cells[idx] as u8;
            }
        }
        count
    }

    // Get the dead and alive values of the entire universe
    pub fn get_cells(&self) -> &[Cell] {
        &self.cells
    }

    // Set cells to be alive in a universe by passing the row and column 
    // of each cell as an array.
    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells[idx] = Cell::Alive;
        }
    }
}

// Gives Universe an implementaton of .to_string()
impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == Cell::Dead { '◻' } else { '◼' };
                write!(f, " {}", symbol)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}

// Public methods, exported to JavaScript.
#[wasm_bindgen]
impl Universe {
    pub fn tick(&mut self) {
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                /*                 
                log!(
                    "cell[{}, {}] is initially {:?} and has {} live neighbors",
                    row,
                    col,
                    cell,
                    live_neighbors
                ); 
                */

                let next_cell = match (cell, live_neighbors) {
                    // Rule 1: Any live cell with fewer than two live neighbors
                    // dies, as if caused by underpopulation.
                    (Cell::Alive, x) if x < 2 => Cell::Dead,
                    // Rule 2: Any live cell with two or three live neighbors
                    // lives on to the next generation.
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    // Rule 3: Any live cells more than three live neighbors
                    // dies, as if by overpopulation
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    // Rule 4: Any dead cell with exactly three live neighbors
                    // becomes a live cell, as if by reproduction.
                    (Cell::Dead, 3) => Cell::Alive,
                    // All other cells remain in the same state.
                    (otherwise, _) =>otherwise
                };

                /*
                log!("    it becomes {:?}", next_cell);
                */

                next[idx] = next_cell;
            }
        }

        self.cells = next;
    }

    // Constructor for a new Universe
    pub fn new() -> Universe {
        
        // Hook for displaying panics in the console.
        utils::set_panic_hook();
        
        let width = 100;
        let height = 100;

        let cells = (0..width * height)
            .map(|i| {
                if i % 2 == 0 || i % 7 == 0 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect();
        
        Universe {
            width,
            height,
            cells
        }
    }

    pub fn random_universe() -> Universe {
        
        let mut rng = rand::thread_rng();

        let width = 100;
        let height = 100;

        let cells = (0..width * height)
            .map(|_| rng.gen_range(0, 2))
            .map(|i| {
                if i == 0 {
                    Cell::Alive
                }
                else {
                    Cell::Dead
                }
            })
            .collect();
        
        Universe {
            width,
            height,
            cells
        }
    }

    // Refactor to use Map
    pub fn kill_universe(&mut self) {
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                self.cells[idx] = Cell::Dead;
            }
        }
    }

    // Creates a pulsar centered at the row / col location.
    pub fn create_pulsar(&mut self, row: u32, column: u32) {
        let y_axis = column;
        let x_axis = row;

        // 0 centered: Seed initial pulsar segment - upper right segment.
        let pulsar_seed = vec![(6,4),(6,3),(6,2),(4,6),(4,1),(3,6),(3,1),(2,6),(2,1),(1,4),(1,3),(1,2)];

        // Map the shape of the upper right pulsar segment to the offset from click location.
        let mut pulsar: Vec<(u32, u32)> =
                        pulsar_seed.iter()
                            .map(|pair| {
                                ((row + pair.0) % self.height , (column + pair.1) % self.width)
                            })
                            .collect();

        // Mirror initial pulsar segment on Y axis.
        let pulsar_segment: Vec<(u32, u32)> = 
                                pulsar.iter()
                                    .map(|pair| {
                                        (pair.0 % self.height, (y_axis + (y_axis - pair.1)) % self.width)
                                    })
                                    .collect();
        
        // Combine mirrored segment with initial segment, resulting in top half of pulsar.
        pulsar.extend(pulsar_segment);
        
        // Mirror top half pulsar segment on X axis.
        let pulsar_segment: Vec<(u32, u32)> = 
                                pulsar.iter()
                                    .map(|pair| {
                                        ((x_axis + (x_axis - pair.0)) % self.height, pair.1 % self.width)
                                    })
                                    .collect();
        
        // Combine top half segment with bottom half segment.
        pulsar.extend(pulsar_segment);

        self.set_cells(&pulsar);
    }

    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.cells[idx].toggle();
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }

    // Sets the width of the universe and resets all cells to the dead state.
    pub fn set_width(&mut self, width: u32) {
        self.width = width;
        self.cells = (0..width * self.height).map(|_i| Cell::Dead).collect();
    }

    // Sets the height of the universe and resets all cells to the dead state.
    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        self.cells = (0..self.width * height).map(|_i| Cell::Dead).collect();
    }

    pub fn render(&self) -> String {
        self.to_string()
    }
}