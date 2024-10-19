use std::fs;

use bevy::prelude::{default, Color};
use noise::{NoiseFn, Perlin};
use rand::*;

const BIRTH_LIMIT: u32 = 5;
const DEATH_LIMIT: u32 = 4;
const N_STEPS: u32 = 4;
const N_COLORS: usize = 12;

const SPRITE_HEIGHT: usize = 45;
const SPRITE_WIDTH: usize = 45;
const CELL_HEIGHT_PX: usize = 8;
const CELL_WIDTH_PX: usize = 8;
const PERLIN_SCALE: f64 = 320.5;

#[derive(Debug)]
pub enum Faction {
    ChaosWarriors,
    WaterBoys,
    ForestBoys,
    TechBoys,
    HellSpawn,
    SpaceAliens,
    GoldenBoys,
    JusticeSoldiers,
}

#[derive(Clone, Copy, Debug)]
pub struct Cell {
    pub position: (i32, i32),
    pub color: (f32, f32, f32, f32),
}

#[derive(Clone, Debug)]
pub struct Group {
    pub arr: Vec<Cell>,
    pub valid: bool,
}

#[derive(Clone, Debug, Default)]
pub struct GroupDrawer {
    pub groups: Vec<Group>,
    pub negative_groups: Vec<Group>,
    pub children: Vec<CellDrawer>,
}

#[derive(Clone, Debug, Default)]
pub struct CellDrawer {
    pub cells: Vec<Cell>,
}

impl GroupDrawer {
    pub fn new(groups: Vec<Group>, negative_groups: Vec<Group>) -> Self {
        GroupDrawer {
            groups,
            negative_groups,
            ..default()
        }
    }

    pub fn add_child(&mut self, cell_drawer: CellDrawer) {
        self.children.push(cell_drawer);
    }

    pub fn get_primary_color(&self) -> Color {
        let mut sum_r: f32 = 0.0;
        let mut sum_g: f32 = 0.0;
        let mut sum_b: f32 = 0.0;

        for c in self.children.iter() {
            for cell in c.cells.iter() {
                let (r, g, b, _) = cell.color;
                sum_r += r;
                sum_g += g;
                sum_b += b;
            }
        }

        if sum_r >= sum_b && sum_r >= sum_b {
            return Color::RED;
        }
        if sum_g >= sum_r && sum_g >= sum_b {
            return Color::GREEN;
        }
        if sum_b >= sum_r && sum_b >= sum_g {
            return Color::BLUE;
        }

        Color::RED
    }

    pub fn get_faction(&self) -> Faction {
        let mut sum_r: f32 = 0.0;
        let mut sum_g: f32 = 0.0;
        let mut sum_b: f32 = 0.0;
        let mut count: f32 = 0.0;

        for c in self.children.iter() {
            for cell in c.cells.iter() {
                let (r, g, b, _) = cell.color;

                if (r == 0.0 && g == 0.0 && b == 0.0) || (r == 255.0 && g == 255.0 && b == 255.0) {
                    continue;
                }

                sum_r += (r * 255.0).round();
                sum_g += (g * 255.0).round();
                sum_b += (b * 255.0).round();
                count += 1.0;
            }
        }

        let avg_r = sum_r / count;
        let avg_g = sum_g / count;
        let avg_b = sum_b / count;

        if avg_r >= 0.0
            && avg_r <= 127.0
            && avg_g >= 0.0
            && avg_g <= 127.0
            && avg_b >= 0.0
            && avg_b <= 127.0
        {
            return Faction::ChaosWarriors;
        }
        if avg_r >= 128.0
            && avg_r <= 255.0
            && avg_g >= 0.0
            && avg_g <= 127.0
            && avg_b >= 0.0
            && avg_b <= 127.0
        {
            return Faction::WaterBoys;
        }
        if avg_r >= 0.0
            && avg_r <= 127.0
            && avg_g >= 128.0
            && avg_g <= 255.0
            && avg_b >= 0.0
            && avg_b <= 127.0
        {
            return Faction::ForestBoys;
        }
        if avg_r >= 128.0
            && avg_r <= 255.0
            && avg_g >= 128.0
            && avg_g <= 255.0
            && avg_b >= 0.0
            && avg_b <= 127.0
        {
            return Faction::TechBoys;
        }
        if avg_r >= 0.0
            && avg_r <= 127.0
            && avg_g >= 0.0
            && avg_g <= 127.0
            && avg_b >= 128.0
            && avg_b <= 255.0
        {
            return Faction::HellSpawn;
        }
        if avg_r >= 128.0
            && avg_r <= 255.0
            && avg_g >= 0.0
            && avg_g <= 127.0
            && avg_b >= 128.0
            && avg_b <= 255.0
        {
            return Faction::SpaceAliens;
        }
        if avg_r >= 0.0
            && avg_r <= 127.0
            && avg_g >= 128.0
            && avg_g <= 255.0
            && avg_b >= 128.0
            && avg_b <= 255.0
        {
            return Faction::GoldenBoys;
        }
        if avg_r >= 128.0
            && avg_r <= 255.0
            && avg_g >= 128.0
            && avg_g <= 255.0
            && avg_b >= 128.0
            && avg_b <= 255.0
        {
            return Faction::JusticeSoldiers;
        }

        Faction::ChaosWarriors
    }

    pub fn ready(&mut self) {
        let mut largest: usize = 0;
        for group in &self.groups {
            largest = max(largest, group.arr.len());
        }

        let group_len = self.groups.len();

        for i in (0..group_len as i32).rev() {
            if let Some(group) = self.groups.get(i as usize) {
                if group.arr.len() as f32 >= largest as f32 * 0.25 {
                    let mut dupe_arr = group.arr.clone();

                    for negative_group in self.negative_groups.iter_mut() {
                        if !negative_group.valid {
                            continue;
                        }

                        if group_is_touching_group(&negative_group, group) {
                            // Overlay negative_group cells ontop of group cells
                            dupe_arr.append(&mut negative_group.arr);
                        }
                    }

                    let cell_drawer = CellDrawer::new(dupe_arr);
                    self.add_child(cell_drawer);
                } else {
                    self.groups.remove(i as usize);
                }
            }
        }
    }

    pub fn draw_all(&self) -> Vec<Vec<(f32, f32, f32, f32)>> {
        let mut board: Vec<Vec<(f32, f32, f32, f32)>> = vec![];
        for _ in 0..SPRITE_HEIGHT {
            let mut row = vec![];
            for _ in 0..SPRITE_WIDTH {
                // Solid white
                row.push((255.0, 255.0, 255.0, 1.0));
            }
            board.push(row);
        }

        for c in self.children.iter() {
            c._draw(&mut board);
        }

        board
    }

    pub fn write_html_file(&self, html_file_path: &str) {
        let mut board_inner_html = vec![];

        for i in 0..self.children.len() {
            let c = &self.children[i];

            let mut group = vec![];
            for cell in c.cells.iter() {
                let (r, g, b, a) = cell.color;
                let top_px = cell.position.1 * CELL_HEIGHT_PX as i32;
                let left_px = cell.position.0 * CELL_WIDTH_PX as i32;

                let div = format!(
                    r#"
                        <div style="position:absolute; top:{}px; left:{}px; height:8px; width:8px; background-color:{};"></div>
                    "#,
                    top_px,
                    left_px,
                    rgba_to_hex((r, g, b, a)),
                );

                group.push(div);
            }

            let group_div = format!(
                r#"
                    <div style="position:absolute" class="group" data-group="{}">
                        <div style="position:relative;">
                            {}
                        </div>
                    </div>
                "#,
                i,
                group.join(""),
            );

            board_inner_html.push(group_div);
        }

        let html = format!(
            r#"
                <!DOCTYPE html>
                <html lang="en">
                <head>
                    <meta charset="UTF-8">
                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                    <title>Sprite</title>
                </head>
                <body>
                    <div id="board" style="position:relative; height:{}px; width:{}px">
                        {}
                    </div>
                    <script src="/sprite_movement.js"></script>
                </body>
                </html>
            "#,
            SPRITE_HEIGHT * CELL_HEIGHT_PX,
            SPRITE_WIDTH * CELL_WIDTH_PX,
            board_inner_html.join(""),
        );

        fs::write(html_file_path, html).unwrap();
    }
}

impl CellDrawer {
    pub fn new(cells: Vec<Cell>) -> Self {
        CellDrawer { cells, ..default() }
    }

    pub fn _draw(&self, board: &mut Vec<Vec<(f32, f32, f32, f32)>>) {
        for c in self.cells.iter() {
            if c.position.0 < 0 && c.position.1 < 0 {
                continue;
            }
            if let Some(row) = board.get(c.position.1 as usize) {
                if let Some(_) = row.get(c.position.0 as usize) {
                    board[c.position.1 as usize][c.position.0 as usize] = c.color;
                }
            }
        }
    }
}

pub fn get_sprite(seed: u32, height: usize, width: usize) -> GroupDrawer {
    let mut map = make_rand_map(height, width);

    cellular_automata_do_steps(&mut map);

    let (groups, negative_groups) = fill_colors(&mut map);

    GroupDrawer::new(groups, negative_groups)
}

pub fn make_rand_map(height: usize, width: usize) -> Vec<Vec<bool>> {
    let mut map = vec![];
    for _ in 0..width {
        map.push(vec![]);
    }

    for x in 0..(width as f32 * 0.5).ceil() as usize {
        let mut arr = vec![];
        for y in 0..height {
            arr.push(rand_bool(0.48));

            // When close to center increase the cances to fill the map, so it's more likely to end up with a sprite that's connected in the middle
            let to_center = ((y as f32 - height as f32 * 0.5).abs() * 2.0) / height as f32;
            if x as f32 == (width as f32 * 0.5).floor() - 1.0
                || x as f32 == (width as f32 * 0.5) - 2.0
            {
                if rand_range(0.0, 0.4) > to_center {
                    arr[y] = true;
                }
            }
        }

        map[x] = arr.clone();
        map[width - x - 1] = arr.clone();
    }

    map
}

pub fn cellular_automata_do_steps(map: &mut Vec<Vec<bool>>) {
    let mut dupe = map.clone();
    for _ in 0..N_STEPS {
        dupe = step(&mut dupe.clone());
    }
    *map = dupe;
}

fn step(map: &Vec<Vec<bool>>) -> Vec<Vec<bool>> {
    let mut dup = map.clone();
    for x in 0..map.len() {
        for y in 0..map[x].len() {
            // Ensure padding of 1 to prevent overflow when border is added later
            if x == 0 || x == map.len() - 1 || y == 0 || y == map[x].len() - 1 {
                dup[x][y] = false;
                continue;
            }

            let cell = dup[x][y];
            let n = get_neighbours(map, (x, y));
            if cell && n < DEATH_LIMIT {
                dup[x][y] = false;
            } else if !cell && n > BIRTH_LIMIT {
                dup[x][y] = true;
            }
        }
    }
    dup
}

fn get_neighbours(map: &Vec<Vec<bool>>, pos: (usize, usize)) -> u32 {
    let mut count = 0;

    for i in -1i32..2 {
        for j in -1i32..2 {
            if i == 0 && j == 0 {
                continue;
            }

            match get_at_pos(map, (pos.0 as i32 + i, pos.1 as i32 + j)) {
                None => continue,
                Some(val) => {
                    if val {
                        count += 1;
                    }
                }
            }
        }
    }

    count
}

fn get_at_pos(map: &Vec<Vec<bool>>, pos: (i32, i32)) -> Option<bool> {
    if pos.0 < 0 || pos.1 < 0 {
        return None;
    }

    if pos.0 < 0
        || pos.0 >= map.len() as i32
        || pos.1 < 0
        || (pos.0 >= 0 && pos.1 >= map[pos.0 as usize].len() as i32)
    {
        return Some(false);
    }

    Some(map[pos.0 as usize][pos.1 as usize])
}

pub fn gen_colorscheme() -> Vec<(f32, f32, f32, f32)> {
    let a = (
        rand_range(0.0, 0.5),
        rand_range(0.0, 0.5),
        rand_range(0.0, 0.5),
    );

    let b = (
        rand_range(0.1, 0.6),
        rand_range(0.1, 0.6),
        rand_range(0.1, 0.6),
    );

    let c = (
        rand_range(0.15, 0.8),
        rand_range(0.15, 0.8),
        rand_range(0.15, 0.8),
    );

    let d = (
        rand_range(0.0, 1.0),
        rand_range(0.0, 1.0),
        rand_range(0.0, 1.0),
    );

    let mut cols = vec![];
    let n = (N_COLORS - 1) as f32;

    for i in 0..N_COLORS {
        let vec3 = (
            // r
            (a.0 + b.0 * (6.28318 * (c.0 * (i as f32 / n) + d.0)).cos()) + (i as f32 / n) * 0.8,
            // g
            (a.1 + b.1 * (6.28318 * (c.1 * (i as f32 / n) + d.1)).cos()) + (i as f32 / n) * 0.8,
            // b
            (a.2 + b.2 * (6.28318 * (c.2 * (i as f32 / n) + d.2)).cos()) + (i as f32 / n) * 0.8,
            // a
            1.0,
        );

        cols.push(vec3);
    }

    cols
}

pub fn fill_colors(map: &mut Vec<Vec<bool>>) -> (Vec<Group>, Vec<Group>) {
    let colorscheme = gen_colorscheme();
    let eye_colorscheme = gen_colorscheme();

    let noise1 = Perlin::new(randu());
    let noise2 = Perlin::new(randu());

    let groups = flood_fill(
        map,
        colorscheme.clone(),
        eye_colorscheme.clone(),
        false,
        noise1,
        noise2,
    );

    let negative_groups = flood_fill_negative(map, colorscheme, eye_colorscheme, noise1, noise2);

    (groups, negative_groups)
}

fn flood_fill_negative(
    map: &mut Vec<Vec<bool>>,
    colorscheme: Vec<(f32, f32, f32, f32)>,
    eye_colorscheme: Vec<(f32, f32, f32, f32)>,
    noise1: Perlin,
    noise2: Perlin,
) -> Vec<Group> {
    let mut negative_map = vec![];
    for x in 0..map.len() {
        let mut arr = vec![];
        for y in 0..map[x].len() {
            if let Some(val) = get_at_pos(map, (x as i32, y as i32)) {
                arr.push(!val);
            }
        }
        negative_map.push(arr);
    }

    flood_fill(
        &mut negative_map,
        colorscheme,
        eye_colorscheme,
        true,
        noise1,
        noise2,
    )
}

fn flood_fill(
    map: &mut Vec<Vec<bool>>,
    colorscheme: Vec<(f32, f32, f32, f32)>,
    eye_colorscheme: Vec<(f32, f32, f32, f32)>,
    is_negative_group: bool,
    noise1: Perlin,
    noise2: Perlin,
) -> Vec<Group> {
    let mut groups: Vec<Group> = vec![];
    let mut checked_map = vec![];
    for x in 0..map.len() {
        let mut arr = vec![];
        for _y in 0..map[x].len() {
            arr.push(false);
        }
        checked_map.push(arr)
    }

    // bucket is all the cells that have been found through flood filling and whose neighbours will be checked next
    let mut bucket: Vec<(i32, i32)> = vec![];
    for x in 0..map.len() {
        for y in 0..map[x].len() {
            // haven't checked this cell yet
            if !checked_map[x][y] {
                checked_map[x][y] = true;
                // if this cell is actually filled in the map
                if map[x][y] {
                    bucket.push((x as i32, y as i32));
                    let mut group = Group {
                        arr: vec![],
                        valid: true,
                    };

                    // go through remaining cells in bucket
                    while bucket.len() > 0 {
                        let pos: (i32, i32) = match bucket.pop() {
                            None => break,
                            Some(p) => p,
                        };
                        // get neighbours
                        let right = get_at_pos(map, (pos.0 + 1, pos.1));
                        let left = get_at_pos(map, (pos.0 - 1, pos.1));
                        let down = get_at_pos(map, (pos.0, pos.1 + 1));
                        let up = get_at_pos(map, (pos.0, pos.1 - 1));
                        // dont want negative groups that touch the edge of the sprite
                        if is_negative_group {
                            if left.is_none() || up.is_none() || down.is_none() || right.is_none() {
                                group.valid = false;
                            }
                        }
                        // also do a coloring step in this flood fill, speeds up processing a bit instead of doing it seperately
                        let col = choose_color(
                            map,
                            pos,
                            is_negative_group,
                            right,
                            left,
                            down,
                            up,
                            colorscheme.clone(),
                            eye_colorscheme.clone(),
                            &mut group,
                            noise1,
                            noise2,
                        );
                        group.arr.push(Cell {
                            position: pos,
                            color: col,
                        });

                        // add neighbours to bucket to check
                        if right.is_some()
                            && right.unwrap()
                            && pos.0 >= 0
                            && pos.1 >= 0
                            && !checked_map[pos.0 as usize + 1][pos.1 as usize]
                        {
                            bucket.push((pos.0 + 1, pos.1));
                            checked_map[(pos.0 + 1) as usize][pos.1 as usize] = true;
                        }
                        if left.is_some()
                            && left.unwrap()
                            && pos.0 - 1 >= 0
                            && pos.1 >= 0
                            && !checked_map[(pos.0 - 1) as usize][pos.1 as usize]
                        {
                            bucket.push((pos.0 - 1, pos.1));
                            checked_map[(pos.0 - 1) as usize][pos.1 as usize] = true;
                        }
                        if down.is_some()
                            && down.unwrap()
                            && pos.0 >= 0
                            && pos.1 + 1 >= 0
                            && !checked_map[pos.0 as usize][(pos.1 + 1) as usize]
                        {
                            bucket.push((pos.0, pos.1 + 1));
                            checked_map[pos.0 as usize][(pos.1 + 1) as usize] = true;
                        }
                        if up.is_some()
                            && up.unwrap()
                            && pos.0 >= 0
                            && pos.1 - 1 >= 0
                            && !checked_map[pos.0 as usize][(pos.1 - 1) as usize]
                        {
                            bucket.push((pos.0, pos.1 - 1));
                            checked_map[pos.0 as usize][(pos.1 - 1) as usize] = true;
                        }
                    }
                    groups.push(group)
                }
            }
        }
    }
    groups
}

fn choose_color(
    map: &mut Vec<Vec<bool>>,
    pos: (i32, i32),
    is_negative_group: bool,
    right: Option<bool>,
    left: Option<bool>,
    down: Option<bool>,
    up: Option<bool>,
    colorscheme: Vec<(f32, f32, f32, f32)>,
    eye_colorscheme: Vec<(f32, f32, f32, f32)>,
    group: &mut Group,
    noise1: Perlin,
    noise2: Perlin,
) -> (f32, f32, f32, f32) {
    let col_x = (pos.0 as f64 - (map.len() - 1) as f64 * 0.5).abs().ceil();
    let mut n1 = (noise1.get([col_x / PERLIN_SCALE, pos.1 as f64 / PERLIN_SCALE]))
        .abs()
        .powf(1.5)
        * 3.0;
    let mut n2 = (noise2.get([col_x / PERLIN_SCALE, pos.1 as f64 / PERLIN_SCALE]))
        .abs()
        .powf(1.5)
        * 3.0;

    // highlight colors based on amount of neighbours
    if down.is_none() || !down.unwrap() {
        if is_negative_group {
            n2 -= 0.1;
        } else {
            n1 -= 0.45;
        }
        n1 *= 0.8;
        group.arr.push(Cell {
            position: (pos.0, pos.1 + 1),
            color: (0.0, 0.0, 0.0, 1.0),
        });
    }
    if right.is_none() || !right.unwrap() {
        if is_negative_group {
            n2 += 0.1;
        } else {
            n1 += 0.2;
        }
        n1 *= 1.1;
        group.arr.push(Cell {
            position: (pos.0 + 1, pos.1),
            color: (0.0, 0.0, 0.0, 1.0),
        });
    }
    if up.is_none() || !up.unwrap() {
        if is_negative_group {
            n2 += 0.15;
        } else {
            n1 += 0.45;
        }
        n1 *= 1.2;
        group.arr.push(Cell {
            position: (pos.0, pos.1 - 1),
            color: (0.0, 0.0, 0.0, 1.0),
        });
    }
    if left.is_none() || !left.unwrap() {
        if is_negative_group {
            n2 += 0.1;
        } else {
            n1 += 0.2;
        }
        n1 *= 1.1;
        group.arr.push(Cell {
            position: (pos.0 - 1, pos.1),
            color: (0.0, 0.0, 0.0, 1.0),
        });
    }
    // highlight colors if the difference in colors between neighbours is big
    let c_0 = colorscheme
        [noise1.get([col_x / PERLIN_SCALE, pos.1 as f64 / PERLIN_SCALE]) as usize * (N_COLORS - 1)];
    let c_1 = colorscheme[noise1.get([col_x / PERLIN_SCALE, (pos.1 - 1) as f64 / PERLIN_SCALE])
        as usize
        * (N_COLORS - 1)];
    let c_2 = colorscheme[noise1.get([col_x / PERLIN_SCALE, (pos.1 + 1) as f64 / PERLIN_SCALE])
        as usize
        * (N_COLORS - 1)];
    let c_3 = colorscheme[noise1.get([(col_x - 1.0) / PERLIN_SCALE, pos.1 as f64 / PERLIN_SCALE])
        as usize
        * (N_COLORS - 1)];
    let c_4 = colorscheme[noise1.get([(col_x + 1.0) / PERLIN_SCALE, pos.1 as f64 / PERLIN_SCALE])
        as usize
        * (N_COLORS - 1)];
    let diff = ((c_0.0 - c_1.0).abs() + (c_0.1 - c_1.1).abs() + (c_0.2 - c_1.2).abs())
        + ((c_0.0 - c_2.0).abs() + (c_0.1 - c_2.1).abs() + (c_0.2 - c_2.2).abs())
        + ((c_0.0 - c_3.0).abs() + (c_0.1 - c_3.1).abs() + (c_0.2 - c_3.2).abs())
        + ((c_0.0 - c_4.0).abs() + (c_0.1 - c_4.1).abs() + (c_0.2 - c_4.2).abs());

    if diff > 2.0 {
        n1 += 0.3;
        n1 *= 1.5;
        n2 += 0.3;
        n2 *= 1.5;
    }

    // choose a color
    n1 = clamp(n1, 0.0, 1.0);
    n1 = (n1 * (N_COLORS as f64 - 1.0)).floor();
    n2 = clamp(n2, 0.0, 1.0);
    n2 = (n2 * (N_COLORS as f64 - 1.0)).floor();
    let mut col = colorscheme[n1 as usize];
    if is_negative_group {
        col = eye_colorscheme[n2 as usize];
    }
    col
}

fn group_is_touching_group(g1: &Group, g2: &Group) -> bool {
    let g1_positions: Vec<(i32, i32)> = g1.arr.iter().map(|c: &Cell| c.position).collect();
    let g2_positions: Vec<(i32, i32)> = g2.arr.iter().map(|c: &Cell| c.position).collect();

    for &(x, y) in &g1_positions {
        if g2_positions.contains(&(x, y)) {
            return true;
        }
    }

    for &(x, y) in &g2_positions {
        if g1_positions.contains(&(x, y)) {
            return true;
        }
    }

    false
}

fn rand_bool(chance: f32) -> bool {
    rand_range(0.0, 1.0) > chance
}

fn rand_range(n1: f32, n2: f32) -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(n1..n2)
}

fn randu() -> u32 {
    rand::thread_rng().gen_range(0..u32::MAX)
}

fn clamp(n: f64, min: f64, max: f64) -> f64 {
    if n > max {
        return max;
    }
    if n < min {
        return min;
    }
    n
}

fn max(n1: usize, n2: usize) -> usize {
    if n1 < n2 {
        return n2;
    }
    n1
}

fn rgba_to_hex(rgba: (f32, f32, f32, f32)) -> String {
    let (r, g, b, a) = rgba;

    // Convert to u8
    let r = (r * 255.0).round() as u8;
    let g = (g * 255.0).round() as u8;
    let b = (b * 255.0).round() as u8;
    let a = (a * 255.0).round() as u8;

    // Create hex string
    format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a)
}
