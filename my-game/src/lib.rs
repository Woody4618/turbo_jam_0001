turbo::cfg! {r#"
    name = "Fajita Frenzy"
    version = "1.0.0"
    author = "Jonas"
    description = "Catch falling pancakes!"
    [settings]
    resolution = [512,320] 
"#}

// Define the game state initialization using the turbo::init! macro
turbo::init! {
    struct GameState {
        frame: u32,
        lost_in_frame: u32,
        fajita_made_in_frame: u32,
        last_munch_at: u32,
        clown_pos_x: f32,
        clown_pos_y: f32,
        clown_r: f32,
        topf_pos_x: f32,
        topf_pos_y: f32,
        topf_slot: f32,
        foods: Vec<struct Food {
            slot_pos_x: f32,
            start_pos_x: f32,
            y: f32,
            fly_up: bool,
            vel: f32,
            radius: f32,
            sprite: String
        }>,
        recipe_collection: Vec<String>,
        score: u32,
        lost_recipe_wrong: bool,
        cannon_burst: u32,
    } = {
        Self {
            frame: 0,
            lost_in_frame: 0,
            fajita_made_in_frame: 99999999,
            last_munch_at: 0,
            clown_pos_x: 128.0,
            clown_pos_y: 112.0,
            topf_pos_x: 128.0,
            topf_pos_y: 112.0,
            clown_r: 32.0,
            topf_slot: 2.0,
            foods: vec![],
            recipe_collection: [].to_vec(),
            score: 0,
            lost_recipe_wrong: false,
            cannon_burst: 99999999,
        }
    }
}

impl GameState {
    pub fn new() -> Self {
        Self {
            frame: 0,
            lost_in_frame: 0,
            fajita_made_in_frame: 99999999,
            last_munch_at: 0,
            clown_pos_x: 128.0,
            clown_pos_y: 112.0,
            topf_pos_x: 128.0,
            topf_pos_y: 112.0,
            clown_r: 32.0,
            topf_slot: 2.0,
            foods: vec![],
            recipe_collection: [].to_vec(),
            score: 0,
            lost_recipe_wrong: false,
            cannon_burst: 99999999,
        }
    }
}

// This is where your main game loop code goes
// The stuff in this block will run ~60x per sec
turbo::go! {
    let mut state = GameState::load();

    let width = 512;
    let height = 320;
    let total_stoves = 5;
    let stove_slot = 60.0;

    sprite!("manegedark", 0, 0);

    if gamepad(0).left.just_released() && state.topf_slot > 0.0 {
        state.topf_slot -= 1.0;
    }
    if gamepad(0).right.just_released() && state.topf_slot + 1.0 < total_stoves as f32 {
        state.topf_slot += 1.0;
    }

    state.clown_pos_x = state.topf_slot * stove_slot;
    state.clown_pos_y = height as f32;

    let food_sprites = ["lauch", "onion", "potatoe", "carrot"];

    // Generate new pancakes at random intervals
    if state.frame % 64 == 0 {
        // Create a new pancake with random attributes
        let food = Food {
            slot_pos_x: ((rand() % 5) as f32 + 1.0)  * stove_slot,
            start_pos_x: ( 2 +total_stoves ) as f32 * stove_slot + 20.0,
            y: 200.0,
            fly_up: true,
            vel: (rand() % 2 + 2) as f32,
            radius: (1) as f32,
            sprite: food_sprites[(rand() % food_sprites.len() as u32) as usize].to_string()
        };
        state.foods.push(food);
        state.cannon_burst = state.frame;
    }

    // Update pancake positions and check for collisions with the cat
    let clown_center = (state.clown_pos_x + 15.0 + state.clown_r, state.clown_pos_y - 80.0 + state.clown_r );

    state.foods.retain_mut(|food| {
        if food.fly_up {
            food.y -= food.vel;
            if food.y < 0.0 {
                food.fly_up = false;
            }
        } else {
            food.y += food.vel;
        }

        // Check for collision with the cat
        let food_center = (food.slot_pos_x + food.radius, food.y + food.radius);

        // Calculate the distance between the cat and the pancake
        let dx = clown_center.0 - food_center.0;
        let dy = clown_center.1 - food_center.1;

        let distance = (dx * dx + dy * dy).sqrt();
        let radii_sum = state.clown_r + food.radius;
        let radii_diff = (state.clown_r - food.radius).abs();

        if radii_diff <= distance && distance <= radii_sum {
            // food caught
            state.score += 1;
            state.last_munch_at = state.frame;

            for allready_colleced in &state.recipe_collection {
                if allready_colleced.to_string().eq(&food.sprite) {
                    state.lost_recipe_wrong = true;
                    state.score = 0;
                    state.lost_in_frame = state.frame;
                }
            }

            state.recipe_collection.push(food.sprite.to_string());

            if (state.recipe_collection.len().eq(&food_sprites.len()) && !state.lost_recipe_wrong) {
                state.fajita_made_in_frame = state.frame;
                state.score += 100;
            }

            false // Remove the food from the game
        } else if food.y < 320. + (food.radius * 2.) {
            true // Keep the pancake in the game if it's within the screen
        } else {
            false // Remove the pancake if it's off-screen
        }
    });

    if state.lost_recipe_wrong && state.lost_in_frame + 120 < state.frame {
        state = GameState::new()
    }

    if state.fajita_made_in_frame + 120 >= state.frame && state.fajita_made_in_frame < state.frame  {
        state.foods = vec![];
        state.recipe_collection = vec![];
        text!(&format!("YOU MADE FAJITA! GoAT. +100points"), x = 40, y = 100, font = Font::L, color = 0xffff0000); // Render the score
    }

    if state.lost_recipe_wrong {
        text!(&format!("Rezept versaut du trottel"), x = 40, y = 60, font = Font::L, color = 0xffff0000); // Render the score
        sprite!("pepe", 40, 80);
        sprite!("pepe", 80, 80);
    }

    text!(&format!("Score: {}", state.score), x = 10, y = 10, font = Font::L, color = 0xffffffff); // Render the score
    text!(&format!("Recipe: Fajitas-> 1 Onion 1Beef 1 Potatoe 1 lauch"), x = 10, y = 40, font = Font::L, color = 0xffffffff); // Render the score

    sprite!("clown",  state.clown_pos_x as i32, state.clown_pos_y as i32 - 145);
    circ!(x = clown_center.0 as i32, y = (clown_center.1) as i32, d = state.clown_r as u32, fill = 0xdba463ff); // Render the pancakes

    //text!(&format!("center {0}", clown_center.1 - 100.0), x = 10, y = 60, font = Font::L, color = 0xffffffff); // Render the score

    sprite!("topf", 34 + state.clown_pos_x as i32, height - 45);

    for i in 1..total_stoves + 1 {
        sprite!("stove", (i as f32 * stove_slot) as i32, height - 10);
    }

    if state.cannon_burst + 4 >= state.frame && state.cannon_burst < state.frame  {
        sprite!("canon", width - 112, height - 112);
    } else {
        sprite!("canon", width - 120, height - 120);
    }

    sprite!("monkey", width - 60, height - 120);

    // Debug
    //text!(&format!("{:#?}", state), y = 24);

    if (state.score < 500) {
        let mut counter = 0;
        for food in &food_sprites {
            let mut isCollected: bool = false;
            for food_collected_food in &state.recipe_collection {
                if food.eq(&food_collected_food.to_string()) {
                    isCollected = true;
                }
            }
            if !isCollected {
                counter += 1;
                sprite!(food, x = 10 ,y = (15.0 + 30.0 * (counter as f32)) as i32);
            }
        }
    } else {
        text!(&format!("hard"), x = 10, y = 120, font = Font::L, color = 0xffff0000);
        text!(&format!("core"), x = 10, y = 128, font = Font::L, color = 0xffff0000);
        text!(&format!("mode"), x = 10, y = 136, font = Font::L, color = 0xffff0000);
    }

    // Draw the falling pancakes
    for food in &state.foods {
        if food.fly_up {
            sprite!(food.sprite.as_str(), x = food.start_pos_x as i32 - 15, y = food.y as i32);
        } else {
            sprite!(food.sprite.as_str(), x = food.slot_pos_x as i32 - 15, y = food.y as i32);
        }
    }

    state.frame += 1;
    state.save();
}
