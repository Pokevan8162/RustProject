//We did not copy code from AI or any other source

use piston_window::*;
use std::f64::consts::PI;

//MAP BUILDING
const MAP_X: usize = 8; //map width
const MAP_Y: usize = 8; //map length
const MAP_S: f64 = 64.0; //each map cube size in pixels
const TEX_SIZE: f64 = 64.0;
const TEX_PER_ROW: f64 = 1.0;

const MAP: [i32; MAP_X * MAP_Y] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 2, 0, 0, 0, 0, 1, 1, 0, 2, 0, 0, 2, 0, 1,
    1, 0, 0, 0, 0, 2, 0, 1, 1, 0, 0, 2, 2, 2, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];

//COLOR DEFINITIONS
const EMPTY_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const PLAYER_COLOR: [f32; 4] = [1.0, 1.0, 0.0, 1.0];
const RAY_COLOR: [f32; 4] = [0.0, 0.8, 0.0, 1.0];

//HELPER FUNCTIONS
//degrees to radians for later functions of cos, sin, tan
fn deg_to_rad(a: f64) -> f64 {
    a * PI / 180.0
}

//keeps player's angle between 0-359 degrees by wrapping around
fn fix_angle(a: f64) -> f64 {
    let mut angle = a;
    if angle > 359.0 {
        angle -= 360.0
    }
    if angle < 0.0 {
        angle += 360.0
    }
    angle
}

// KEY PRESSING STRUCTURE
struct Pressed {
    w: bool,
    a: bool,
    s: bool,
    d: bool,
}
impl Pressed {
    fn new() -> Self {
        Self {
            w: false,
            a: false,
            s: false,
            d: false,
        }
    }
}

//PLAYER STRUCTURE
struct Player {
    px: f64,  //player position x in (x, y)
    py: f64,  //player position x in (x, y)
    pdx: f64, //x component of facing direction vecotr
    pdy: f64, //y component of facing direction vector
    pa: f64,  //the facing angle of player
}

impl Player {
    fn new() -> Self {
        let pa: f64 = 90.0; //initial angle (facing upward)
        let pdx = pa.deg_to_rad().cos(); //x-component of facing direction
        let pdy = -pa.deg_to_rad().sin(); //y-component of facing direction (negative because y increases down)
        let px = 150.0; //initial x position of player
        let py = 400.0; //initial y position of player
        Self {
            px,
            py,
            pdx,
            pdy,
            pa,
        }
    }

    fn draw(&self, transform: math::Matrix2d, g: &mut G2d) {
        let rect = [self.px as f64 - 4.0, self.py as f64 - 4.0, 8.0, 8.0];
        rectangle(PLAYER_COLOR, rect, transform, g); //draws player as yellow rectangle centered on px, py

        let x2 = self.px + self.pdx * 20.0; // calculates where to have line extend to
        let y2 = self.py + self.pdy * 20.0; // calculates where to have line extend to
        line(
            PLAYER_COLOR,
            2.0f64,
            [self.px, self.py, x2, y2],
            transform,
            g,
        ); // extends line 20px from player position and direction vector
    }

    //uses keys a, d, w, s to move the position of the player
    // (w and s move forward or backward (according to direction). a and d rotate angle
    fn update(&mut self, pressed: &Pressed, dt: f64) {
        // rotate
        if pressed.a {
            self.pa += 2.0 * 60.0 * dt;
        }
        if pressed.d {
            self.pa -= 2.0 * 60.0 * dt;
        }

        self.pa = fix_angle(self.pa);

        let rad = deg_to_rad(self.pa);
        self.pdx = rad.cos();
        self.pdy = -rad.sin();

        // move 2 px
        let speed = 2.0 * 60.0;

        //W and S move forward or backward 5 pixels
        if pressed.w {
            self.px += self.pdx * speed * dt;
            self.py += self.pdy * speed * dt;
            let x = (self.px / 64.0) as usize;
            let y = (self.py / 64.0) as usize;
            let index = y * MAP_X + x;

            if MAP[index] != 0 {
                self.px -= self.pdx * speed * dt;
                self.py -= self.pdy * speed * dt;
            }
            // Print the array index of player location
            // println!("Player X: {}", (self.px / 64.0) as i32);
            // println!("Player Y: {}", (self.py / 64.0) as i32);
        }
        if pressed.s {
            self.px -= self.pdx * speed * dt;
            self.py -= self.pdy * speed * dt;

            let x = (self.px / 64.0) as usize;
            let y = (self.py / 64.0) as usize;
            let index = y * MAP_X + x;

            if MAP[index] != 0 {
                self.px += self.pdx * speed * dt;
                self.py += self.pdy * speed * dt;
            }
        }
    }

    fn draw_rays(
        &self,
        transform: math::Matrix2d,
        g: &mut G2d,
        tex1: &G2dTexture,
        tex2: &G2dTexture,
    ) {
        let mut ra = fix_angle(self.pa + 30.0); //sets player field view to 60 degrees (30 degrees right 30 degrees left)

        //iterate each degree of player field view to draw ray
        let ray_count = 60;
        for r in 0..ray_count {
            let mut rx; //ray x-coordinate (x position of ray)
            let mut ry: f64; //ray y-coordinate (y position of ray)
            let mut xo; //x step increment (how much ray moves in x-direction each iteration)
            let mut yo; //y step increment (hocar much ray moves in y-direction each iteration)
            let mut dis_v = 100000.0; //distance to vertical wall hit
            let mut dis_h = 100000.0; //distance to horizontal wall hit
            let mut vx = 0.0; //x-coordinate of vertical intersection (point where ray hits vertical wall)
            let mut vy = 0.0; //y-coordinate of vertical intersection (point where ray hits vertical wall)

            let tan_ra = (deg_to_rad(ra)).tan(); //tangent of current ray angle (slope of the ray)

            //vertical wall hit (where does first vertical grid line get intersected by ray)

            let mut dof = 0; //depth of field (number of grid steps)

            if (deg_to_rad(ra)).cos() > 0.001 {
                //Case: Ray points toward +X (right)
                rx = (((self.px as i32 >> 6) << 6) + 64) as f64; //calculates the x of the first vertical grid boundary to the right of player
                ry = (self.px - rx) * tan_ra + self.py; //calculates y coordinate at that rx
                xo = 64.0; //next step (x)
                yo = -xo * tan_ra; //next step (y)
            } else if (deg_to_rad(ra)).cos() < -0.001 {
                //Case: Ray points toward -X (left)
                rx = (((self.px as i32 >> 6) << 6) as f64) - 0.0001; //caclculates the x of the first vertical grid boundary to the left of player
                ry = (self.px - rx) * tan_ra + self.py; //calculates y coordinate at that rx
                xo = -64.0; //next step (x)
                yo = -xo * tan_ra; //next step (y)
            } else {
                //Case: Ray pointing almost exactly vertical ()
                rx = self.px;
                ry = self.py;
                dof = 8;
                xo = 0.0;
                yo = 0.0;
            }

            while dof < 8 {
                //converts (rx, ry) to map grid coord (mx, my)
                let mx: i32 = (rx / 64.0) as i32;
                let my: i32 = (ry / 64.0) as i32;
                let mp: i32 = my * MAP_X as i32 + mx;

                //checks if ray hits a cell that is a wall (when MAP[] == 1)
                //if it does you stop tracing and compute dis_v
                //if not, step forward on ray
                if mp >= 0 && mp < (MAP_X * MAP_Y) as i32 && MAP[mp as usize] != 0 {
                    dof = 8;
                    dis_v = (deg_to_rad(ra)).cos() * (rx - self.px)
                        - (deg_to_rad(ra)).sin() * (ry - self.py);
                    // if r == rayCount/2 {
                    // println!("Distance Vertical for ray {}: {}", r, dis_v);
                    // }
                } else {
                    rx += xo;
                    ry += yo;
                    dof += 1;
                }
            }
            vx = rx;
            vy = ry;

            //horizontal wall hit where does first horizontal grid line get intersected by ray
            dof = 0;
            let tan_ra: f64 = 1.0 / tan_ra;

            if (deg_to_rad(ra)).sin() > 0.001 {
                //Case 1: ray is pointing up
                ry = (((self.py as i32 >> 6) << 6) as f64) - 0.0001; //calculates the y of the first horizontal grid boundary above the player
                rx = (self.py - ry) * tan_ra + self.px; //calculates x at that calculation of ry
                yo = -64.0; //next step (x)
                xo = -yo * tan_ra; //next step (y)
            } else if (deg_to_rad(ra)).sin() < -0.001 {
                //Case 2: ray is pointing down
                ry = (((self.py as i32 >> 6) << 6) + 64) as f64; //calculates the y of the first horizontal grid boundary below the player
                rx = (self.py - ry) * tan_ra + self.px; //calculates the x at that calulated ry
                yo = 64.0; //next step (x)
                xo = -yo * tan_ra; //next step (y)
            } else {
                //Case 3: ray is  pointing nearly horizontal
                rx = self.px;
                ry = self.py;
                dof = 8;
                xo = 0.0;
                yo = 0.0;
            }

            while dof < 8 {
                //converts (rx, ry) to map grid coord (mx, my)
                let mx: i32 = (rx / 64.0) as i32;
                let my: i32 = (ry / 64.0) as i32;
                let mp: i32 = my * MAP_X as i32 + mx;

                //checks if ray hits a cell that is a wall (when MAP[] == 1)
                //if it does you stop tracing and compute dis_v
                //if not, step forward on ray
                if mp >= 0 && mp < (MAP_X * MAP_Y) as i32 && MAP[mp as usize] != 0 {
                    dof = 8;
                    dis_h = (deg_to_rad(ra)).cos() * (rx - self.px)
                        - (deg_to_rad(ra)).sin() * (ry - self.py);
                    // if r == rayCount/2 {
                    //     println!("Distance Horizontal for ray {}: {}", r, dis_h);
                    // }
                } else {
                    rx += xo;
                    ry += yo;
                    dof += 1;
                }
            }

            //evaluate which hit is closer (vertical vs. horizontal distance)
            let (final_rx, final_ry, dist) = if dis_v < dis_h {
                (vx, vy, dis_v)
            } else {
                (rx, ry, dis_h)
            };

            //Draws 2D line from player to wall
            line(
                RAY_COLOR,
                2.0f64,
                [self.px, self.py, final_rx, final_ry],
                transform,
                g,
            );

            //Draw 3D Projection
            let corrected_dist = dist * (deg_to_rad(self.pa - ra)).cos(); //fisheye correction (limits the skewing)
            let line_h = (MAP_S * 320.0) / corrected_dist; //map size times screed height for scaling then nearby walls produce tall columns, far produce short
            let line_off = 512.0 / 2.0 - line_h / 2.0; //center the wall vertically

            //Each ray maps to one vertical column in the 3D view (right side of the screen)
            let wall_x = 530.0 + (r as f64) * 8.0; //530 is the left edge offset (where 3d view starts) so this shifts wall column to right so each ray has own strip

            let mx = (final_rx / MAP_S) as i32;
            let my = (final_ry / MAP_S) as i32;
            let tile = MAP[(my * MAP_X as i32 + mx) as usize];
            let tex_x = 0.0;
            let tex_y = 0.0;

            let hit_offset = if dis_v < dis_h {
                final_ry % TEX_SIZE
            } else {
                final_rx % TEX_SIZE
            };

            let src = [hit_offset, 0.0, 1.0, TEX_SIZE];
            let tex = if tile == 1 { tex1 } else { tex2 };

            Image::new().src_rect(src).draw(
                tex,
                &DrawState::default(),
                transform
                    .trans(wall_x, line_off)
                    .scale(8.0, line_h / TEX_SIZE),
                g,
            );

            //step tpo next ray angle
            ra = fix_angle(ra - 1.0);
        }
    }
}

fn main() {
    let mut window: PistonWindow = WindowSettings::new("Rust Raycaster", [1024, 512])
        .exit_on_esc(true)
        .build()
        .unwrap();

    // TEXTURE PATHS
    let bricks_tex = Texture::from_path(
        &mut window.create_texture_context(),
        "assets/bricks.png",
        Flip::None,
        &TextureSettings::new(),
    )
    .unwrap();

    let bricks2 = Texture::from_path(
        &mut window.create_texture_context(),
        "assets/bricks2.png",
        Flip::None,
        &TextureSettings::new(),
    )
    .unwrap();

    let mut player = Player::new();
    let mut pressed = Pressed::new();

    while let Some(event) = window.next() {
        // KEY PRESS
        if let Some(Button::Keyboard(key)) = event.press_args() {
            match key {
                Key::W => pressed.w = true,
                Key::S => pressed.s = true,
                Key::A => pressed.a = true,
                Key::D => pressed.d = true,
                _ => {}
            }
        }

        // KEY RELEASE
        if let Some(Button::Keyboard(key)) = event.release_args() {
            match key {
                Key::W => pressed.w = false,
                Key::S => pressed.s = false,
                Key::A => pressed.a = false,
                Key::D => pressed.d = false,
                _ => {}
            }
        }

        if let Some(u) = event.update_args() {
            player.update(&pressed, u.dt);
        }

        //draw map
        window.draw_2d(&event, |c, g, _| {
            clear([0.3, 0.3, 0.3, 1.0], g);

            for y in 0..MAP_Y {
                for x in 0..MAP_X {
                    let xo = x as f64 * MAP_S;
                    let yo = y as f64 * MAP_S;

                    let tile = MAP[y * MAP_X + x];

                    // set color based on number in matrix
                    let color = match tile {
                        1 => [1.0, 0.0, 0.0, 1.0], // red
                        2 => [0.0, 0.0, 1.0, 1.0], // blue
                        _ => EMPTY_COLOR,
                    };
                    rectangle(color, [xo, yo, MAP_S, MAP_S], c.transform, g);
                }
            }

            player.draw(c.transform, g);
            player.draw_rays(c.transform, g, &bricks_tex, &bricks2
    );
        });
    }
}
