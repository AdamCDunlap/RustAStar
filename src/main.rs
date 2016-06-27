extern crate sfml;

use sfml::window::{ContextSettings, VideoMode, event, window_style};
use sfml::graphics::{RenderWindow, RenderTarget, Color, Transformable, Shape, RectangleShape};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::cmp;

const CELL_SIZE : usize = 50;

struct AStarNode {
    loc: (usize, usize),
    score: usize,
}

impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score.cmp(&other.score).reverse()
    }
}

impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other : &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for AStarNode {
    fn eq(&self, other : &Self) -> bool {
        self.loc.eq(&other.loc)
    }
}
impl Eq for AStarNode {}

impl Hash for AStarNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.loc.hash(state);
    }
}

#[derive(PartialEq, Clone, Copy)]
enum CellType {
    Wall, Unvisited, Visited, Fixed, Path
}

// Implements A*, but every time you call expand_one it just does one step
struct SlowAStar {
    dest: (usize, usize),
    open_queue: BinaryHeap<AStarNode>,
    open_set: HashSet<(usize,usize)>,
    closed_set: HashSet<(usize,usize)>,
    dists_from_start: HashMap<(usize,usize), usize>,
    optimal_parents: HashMap<(usize,usize), (usize,usize)>,
    map_picture: Vec<Vec<CellType>>,
}

impl SlowAStar {
    fn score_point(pt: (usize,usize), dist_from_start: usize,
                   dest: (usize,usize)) -> usize {
        dist_from_start + (if pt.0 < dest.0 { dest.0 - pt.0 } else { pt.0 - dest.0 })
                        + (if pt.1 < dest.1 { dest.1 - pt.1 } else { pt.1 - dest.1 })
        //dist_from_start + cmp::max((if pt.0 < dest.0 { dest.0 - pt.0 } else { pt.0 - dest.0 }),
        //                           (if pt.1 < dest.1 { dest.1 - pt.1 } else { pt.1 - dest.1 }))
    }

    // Returns if it's done
    fn expand_one(&mut self) -> bool {
        let mut lowest_node;
        loop {
            lowest_node = match self.open_queue.pop() {
                Some(x) => x,
                None => panic!("No path from start to end")
            };
            if !self.closed_set.contains(&lowest_node.loc) {
                break;
            }
        }
        let lowest_node = lowest_node;
        let lowest = lowest_node.loc;

        assert!(self.map_picture[lowest.0][lowest.1] != CellType::Fixed);
        self.map_picture[lowest.0][lowest.1] = CellType::Fixed;

        if lowest == self.dest {
            let mut pathpt = lowest;
            loop {
                self.map_picture[pathpt.0][pathpt.1] = CellType::Path;
                if self.dists_from_start[&pathpt] == 0 {
                    break;
                }
                pathpt = self.optimal_parents[&pathpt];
            }
            return true;
        }

        let lowest_dist_from_start = self.dists_from_start[&lowest];

        for xp in -1..2 {
            for yp in -1..2 {
                // Only check the four nondiagonal movements
                if (xp == 0) == (yp == 0) { continue; }

                let p = (lowest.0 as i32 + xp, lowest.1 as i32 + yp);
                if p.0 >= 0 && p.1 >= 0
                            && p.0 < self.map_picture.len() as i32
                            && p.1 < self.map_picture[0].len() as i32 {
                    let p = (p.0 as usize, p.1 as usize);

                    if self.map_picture[p.0][p.1] != CellType::Wall
                            && !self.closed_set.contains(&p) {

                        let dist_from_start = lowest_dist_from_start+1;

                        let is_best_path_to_p = if self.open_set.insert(p) {
                            true
                        } else {
                            // p was already in the map, check if this path to
                            // it is better
                            dist_from_start < self.dists_from_start[&p]
                        };

                        if is_best_path_to_p {
                            self.map_picture[p.0][p.1] = CellType::Visited;


                            let score = SlowAStar::score_point(p, dist_from_start, self.dest);

                            let n = AStarNode{loc: p, score: score};

                            self.open_queue.push(n);
                            self.optimal_parents.insert(p, lowest);
                            self.dists_from_start.insert(p, dist_from_start);
                        }
                    }
                }
            }
        }

        self.open_set.remove(&lowest);
        self.closed_set.insert(lowest);
        false
    }

    fn get_map_picture(&self) -> &Vec<Vec<CellType>> { &self.map_picture }

    fn new(map: Vec<Vec<bool>>, start: (usize,usize), dest: (usize,usize)) -> SlowAStar {


        let optimal_parents = HashMap::new();

        let mut dists_from_start = HashMap::new();
        dists_from_start.insert(start, 0);

        let mut open_set: HashSet<(usize,usize)> = HashSet::new();
        open_set.insert(start);

        let mut open_queue: BinaryHeap<AStarNode> = BinaryHeap::new();
        open_queue.push(AStarNode{ loc: start, score: SlowAStar::score_point(start, 0, dest)});



        let closed_set = HashSet::new();

        let mut map_picture = Vec::with_capacity(map.len());
        for in_row in map {
            let mut row = Vec::with_capacity(in_row.len());
            for cell in in_row {
                row.push(if cell {CellType::Unvisited} else {CellType::Wall} );
            }
            map_picture.push(row);
        }
        SlowAStar { dest: dest, open_queue: open_queue, open_set: open_set,
            closed_set: closed_set, dists_from_start: dists_from_start,
            optimal_parents: optimal_parents, map_picture: map_picture }
    }
}

fn main() {
    let map = vec![
        vec![ false, false, false, false, false, false, false, false, false, false, false, false, ],
        vec![ false,  true,  true,  true,  true,  true,  true,  true,  true,  true,  true, false, ],
        vec![ false,  true,  true,  true,  true,  true,  true,  true, false, false,  true, false, ],
        vec![ false,  true,  true,  true,  true,  true,  true, false,  true,  true,  true, false, ],
        vec![ false,  true,  true,  true,  true,  true,  true, false,  true, false, false, false, ],
        vec![ false,  true, false,  true,  true,  true,  true, false,  true,  true,  true, false, ],
        vec![ false,  true,  true,  true, false,  true,  true, false, false, false,  true, false, ],
        vec![ false,  true,  true,  true, false, false, false, false,  true,  true,  true, false, ],
        vec![ false,  true,  true,  true, false,  true,  true,  true,  true, false, false, false, ],
        vec![ false,  true,  true,  true, false,  true, false, false,  true,  true,  true, false, ],
        vec![ false,  true,  true,  true,  true,  true,  true,  true, false,  true,  true, false, ],
        vec![ false, false, false, false, false, false, false, false, false, false, false, false, ],
    ];

    // Create the window of the application
    let mut window = match RenderWindow::new(
             VideoMode::new_init((CELL_SIZE*map.len()) as u32, (CELL_SIZE*map[0].len()) as u32, 32),
             "Map",
             window_style::CLOSE,
             &ContextSettings::default()) {
        Some(window) => window,
        None => panic!("Cannot create a new Render Window.")
    };

    let mut alg = SlowAStar::new(map, (2,2), (10, 10));
    let mut done = false;
    while window.is_open() {
        // Handle events
        for event in window.events() {
            match event {
                event::Closed => window.close(),
                event::KeyPressed{..} => {
                    if done {
                        window.close();
                    } else {
                        done = alg.expand_one();
                    }
                },
                _             => {/* do nothing */}
            }
        }

        let a_star_progress_grid = alg.get_map_picture();

        // Clear the window
        window.clear(&Color::white());
        for (row_idx, row) in a_star_progress_grid.iter().enumerate() {
            for (col_idx, cell) in row.iter().enumerate(){
                let mut rect = match RectangleShape::new() {
                    Some(r) => r,
                    None    => panic!("Cannot create rectangle"),
                };
                rect.set_position2f((col_idx*50) as f32, (row_idx*50) as f32);
                rect.set_size2f(50 as f32, 50 as f32);
                rect.set_fill_color( &match *cell {
                        CellType::Wall => Color::blue(),
                        CellType::Unvisited => Color::white(),
                        CellType::Visited => Color::magenta(),
                        CellType::Fixed => Color::red(),
                        CellType::Path => Color::green(),
                });
                rect.set_outline_color(&Color::cyan());
                rect.set_outline_thickness(1.0);

                let rect = rect;
                window.draw(&rect);
            }
        }

        // Display things on screen
        window.display()
    }
}
