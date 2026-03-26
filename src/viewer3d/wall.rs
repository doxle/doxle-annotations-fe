use serde::Serialize;

/// A framed wall panel
#[derive(Clone, Debug, Serialize)]
pub struct Wall {
    /// Wall length in meters
    pub length: f64,
    /// Wall height in meters
    pub height: f64,
    /// Stud/plate width in meters (e.g. 0.090 for 90mm)
    pub stud_width: f64,
    /// Stud/plate depth in meters (e.g. 0.045 for 45mm)
    pub stud_depth: f64,
    /// Plate height in meters (same as stud_width typically)
    pub plate_height: f64,
}

/// A single framing member (stud or plate) positioned in 3D space
#[derive(Clone, Debug, Serialize)]
pub struct FramingMember {
    pub name: String,
    pub member_type: String,
    /// Center position
    pub x: f64,
    pub y: f64,
    pub z: f64,
    /// Box dimensions (Three.js BoxGeometry)
    pub width: f64,
    pub height: f64,
    pub depth: f64,
    /// Hex color
    pub color: u32,
    /// Rotation axis for pipes: 0=along X, 1=along Z
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation_axis: Option<u8>,
}

/// JSON wrapper sent to Three.js
#[derive(Clone, Debug, Serialize)]
pub struct WallViewerData {
    pub members: Vec<FramingMember>,
}

const STUD_COLOR: u32 = 0xDDBB66; // warm wood
const PLATE_COLOR: u32 = 0xAA8844; // darker wood for plates
const NOG_COLOR: u32 = 0xCC9944; // nogging

impl Wall {
    /// Create a standard 90x45 timber wall
    pub fn standard(length: f64, height: f64) -> Self {
        Self {
            length,
            height,
            stud_width: 0.090,
            stud_depth: 0.045,
            plate_height: 0.045,
        }
    }

    /// Generate all framing members for this wall
    pub fn generate_framing(&self) -> WallViewerData {
        let mut members = Vec::new();
        let stud_spacing = 0.450; // 450mm centers

        // Bottom plate: sits on ground (y = half plate height)
        members.push(FramingMember {
            name: "Bottom Plate".into(),
            member_type: "plate".into(),
            x: self.length / 2.0,
            y: self.plate_height / 2.0,
            z: 0.0,
            width: self.length,
            height: self.plate_height,
            depth: self.stud_width,
            color: PLATE_COLOR,
            rotation_axis: None,
        });

        // Top plate 1: at top of studs
        let top_plate_y = self.height - self.plate_height * 1.5;
        members.push(FramingMember {
            name: "Top Plate 1".into(),
            member_type: "plate".into(),
            x: self.length / 2.0,
            y: top_plate_y,
            z: 0.0,
            width: self.length,
            height: self.plate_height,
            depth: self.stud_width,
            color: PLATE_COLOR,
            rotation_axis: None,
        });

        // Top plate 2: on top of first top plate
        let top_plate2_y = self.height - self.plate_height / 2.0;
        members.push(FramingMember {
            name: "Top Plate 2".into(),
            member_type: "plate".into(),
            x: self.length / 2.0,
            y: top_plate2_y,
            z: 0.0,
            width: self.length,
            height: self.plate_height,
            depth: self.stud_width,
            color: PLATE_COLOR,
            rotation_axis: None,
        });

        // Studs: between bottom plate and first top plate
        let stud_bottom = self.plate_height;
        let stud_top = self.height - self.plate_height * 2.0;
        let stud_height = stud_top - stud_bottom;
        let stud_center_y = stud_bottom + stud_height / 2.0;

        // Number of spaces
        let num_spaces = (self.length / stud_spacing).floor() as usize;

        for i in 0..=num_spaces {
            let x = (i as f64) * stud_spacing;
            // Clamp last stud to wall end
            let x = if x > self.length { self.length } else { x };

            members.push(FramingMember {
                name: format!("Stud {}", i + 1),
                member_type: "stud".into(),
                x,
                y: stud_center_y,
                z: 0.0,
                width: self.stud_depth,  // 45mm face
                height: stud_height,
                depth: self.stud_width,  // 90mm depth
                color: STUD_COLOR,
                rotation_axis: None,
            });
        }

        // End stud at exact wall length (if not already placed)
        let last_stud_x = (num_spaces as f64) * stud_spacing;
        if (self.length - last_stud_x).abs() > 0.01 {
            members.push(FramingMember {
                name: format!("Stud {}", num_spaces + 2),
                member_type: "stud".into(),
                x: self.length,
                y: stud_center_y,
                z: 0.0,
                width: self.stud_depth,
                height: stud_height,
                depth: self.stud_width,
                color: STUD_COLOR,
                rotation_axis: None,
            });
        }

        // Nogging: horizontal member at mid-height between each pair of studs
        let nog_y = stud_center_y;
        for i in 0..num_spaces {
            let x1 = (i as f64) * stud_spacing;
            let x2 = ((i + 1) as f64) * stud_spacing;
            let x2 = if x2 > self.length { self.length } else { x2 };
            let nog_x = (x1 + x2) / 2.0;
            let nog_width = x2 - x1 - self.stud_depth; // fit between studs

            if nog_width > 0.01 {
                members.push(FramingMember {
                    name: format!("Nog {}", i + 1),
                    member_type: "nog".into(),
                    x: nog_x,
                    y: nog_y,
                    z: 0.0,
                    width: nog_width,
                    height: self.stud_depth,  // 45mm tall
                    depth: self.stud_width,   // 90mm deep
                    color: NOG_COLOR,
                    rotation_axis: None,
                });
            }
        }

        WallViewerData { members }
    }

    /// Serialize framing data to JSON for Three.js
    pub fn to_viewer_json(&self) -> String {
        let data = self.generate_framing();
        serde_json::to_string(&data).unwrap_or_default()
    }
}

const WINDOW_COLOR: u32 = 0x88CCEE; // light blue glass tint
const HEADER_COLOR: u32 = 0x997744; // darker for structural header

/// Generate wall framing with a window opening
fn generate_wall_with_window(
    wall: &Wall,
    win_x: f64,     // left edge of window from wall origin
    win_sill: f64,  // sill height from ground
    win_w: f64,     // window width
    win_h: f64,     // window height
) -> WallViewerData {
    let mut members = Vec::new();
    let stud_spacing = 0.450;

    // Plates (same as normal wall)
    members.push(FramingMember {
        name: "Bottom Plate".into(), member_type: "plate".into(),
        x: wall.length / 2.0, y: wall.plate_height / 2.0, z: 0.0,
        width: wall.length, height: wall.plate_height, depth: wall.stud_width,
        color: PLATE_COLOR, rotation_axis: None,
    });
    let top_plate_y = wall.height - wall.plate_height * 1.5;
    members.push(FramingMember {
        name: "Top Plate 1".into(), member_type: "plate".into(),
        x: wall.length / 2.0, y: top_plate_y, z: 0.0,
        width: wall.length, height: wall.plate_height, depth: wall.stud_width,
        color: PLATE_COLOR, rotation_axis: None,
    });
    let top_plate2_y = wall.height - wall.plate_height / 2.0;
    members.push(FramingMember {
        name: "Top Plate 2".into(), member_type: "plate".into(),
        x: wall.length / 2.0, y: top_plate2_y, z: 0.0,
        width: wall.length, height: wall.plate_height, depth: wall.stud_width,
        color: PLATE_COLOR, rotation_axis: None,
    });

    let stud_bottom = wall.plate_height;
    let stud_top = wall.height - wall.plate_height * 2.0;
    let stud_height = stud_top - stud_bottom;
    let stud_center_y = stud_bottom + stud_height / 2.0;

    let win_right = win_x + win_w;
    let win_head = win_sill + win_h; // top of window opening

    // Regular studs — skip any that fall inside the window opening
    let num_spaces = (wall.length / stud_spacing).floor() as usize;
    for i in 0..=num_spaces {
        let x = ((i as f64) * stud_spacing).min(wall.length);
        let in_opening = x > (win_x + wall.stud_depth / 2.0) && x < (win_right - wall.stud_depth / 2.0);
        if !in_opening {
            members.push(FramingMember {
                name: format!("Stud {}", i + 1), member_type: "stud".into(),
                x, y: stud_center_y, z: 0.0,
                width: wall.stud_depth, height: stud_height, depth: wall.stud_width,
                color: STUD_COLOR, rotation_axis: None,
            });
        }
    }
    // End stud
    let last_x = (num_spaces as f64) * stud_spacing;
    if (wall.length - last_x).abs() > 0.01 {
        members.push(FramingMember {
            name: "End Stud".into(), member_type: "stud".into(),
            x: wall.length, y: stud_center_y, z: 0.0,
            width: wall.stud_depth, height: stud_height, depth: wall.stud_width,
            color: STUD_COLOR, rotation_axis: None,
        });
    }

    // King studs (full height, at window edges)
    members.push(FramingMember {
        name: "King Stud L".into(), member_type: "stud".into(),
        x: win_x, y: stud_center_y, z: 0.0,
        width: wall.stud_depth, height: stud_height, depth: wall.stud_width,
        color: STUD_COLOR, rotation_axis: None,
    });
    members.push(FramingMember {
        name: "King Stud R".into(), member_type: "stud".into(),
        x: win_right, y: stud_center_y, z: 0.0,
        width: wall.stud_depth, height: stud_height, depth: wall.stud_width,
        color: STUD_COLOR, rotation_axis: None,
    });

    // Trimmer studs (bottom plate to header, inside king studs)
    let trimmer_height = win_head - stud_bottom;
    let trimmer_cy = stud_bottom + trimmer_height / 2.0;
    members.push(FramingMember {
        name: "Trimmer L".into(), member_type: "stud".into(),
        x: win_x + wall.stud_depth, y: trimmer_cy, z: 0.0,
        width: wall.stud_depth, height: trimmer_height, depth: wall.stud_width,
        color: STUD_COLOR, rotation_axis: None,
    });
    members.push(FramingMember {
        name: "Trimmer R".into(), member_type: "stud".into(),
        x: win_right - wall.stud_depth, y: trimmer_cy, z: 0.0,
        width: wall.stud_depth, height: trimmer_height, depth: wall.stud_width,
        color: STUD_COLOR, rotation_axis: None,
    });

    // Header (lintel) across top of window
    let header_depth = wall.stud_depth * 2.0; // double header
    members.push(FramingMember {
        name: "Header".into(), member_type: "header".into(),
        x: win_x + win_w / 2.0, y: win_head + header_depth / 2.0, z: 0.0,
        width: win_w + wall.stud_depth * 2.0, height: header_depth, depth: wall.stud_width,
        color: HEADER_COLOR, rotation_axis: None,
    });

    // Sill
    members.push(FramingMember {
        name: "Sill".into(), member_type: "plate".into(),
        x: win_x + win_w / 2.0, y: win_sill, z: 0.0,
        width: win_w, height: wall.plate_height, depth: wall.stud_width,
        color: PLATE_COLOR, rotation_axis: None,
    });

    // Cripple studs below sill
    let cripple_h = win_sill - stud_bottom - wall.plate_height / 2.0;
    if cripple_h > 0.05 {
        let cripple_cy = stud_bottom + cripple_h / 2.0;
        let mut cx = win_x + stud_spacing;
        while cx < win_right - wall.stud_depth {
            members.push(FramingMember {
                name: "Cripple".into(), member_type: "stud".into(),
                x: cx, y: cripple_cy, z: 0.0,
                width: wall.stud_depth, height: cripple_h, depth: wall.stud_width,
                color: STUD_COLOR, rotation_axis: None,
            });
            cx += stud_spacing;
        }
    }

    // Glass pane (visual only)
    members.push(FramingMember {
        name: "Window Glass".into(), member_type: "glass".into(),
        x: win_x + win_w / 2.0, y: win_sill + win_h / 2.0, z: 0.0,
        width: win_w, height: win_h, depth: 0.006,
        color: WINDOW_COLOR, rotation_axis: None,
    });

    // Nogging in solid sections (skip window zone)
    let nog_y = stud_center_y;
    for i in 0..num_spaces {
        let x1 = (i as f64) * stud_spacing;
        let x2 = (((i + 1) as f64) * stud_spacing).min(wall.length);
        let nog_center = (x1 + x2) / 2.0;
        // Skip if nogging would be in window zone
        if nog_center > win_x && nog_center < win_right { continue; }
        let nog_width = x2 - x1 - wall.stud_depth;
        if nog_width > 0.01 {
            members.push(FramingMember {
                name: format!("Nog {}", i + 1), member_type: "nog".into(),
                x: nog_center, y: nog_y, z: 0.0,
                width: nog_width, height: wall.stud_depth, depth: wall.stud_width,
                color: NOG_COLOR, rotation_axis: None,
            });
        }
    }

    WallViewerData { members }
}

/// Generate 4 walls forming a room, centered at origin
pub fn generate_room(room_length: f64, room_depth: f64, wall_height: f64) -> String {
    let long_wall = Wall::standard(room_length, wall_height);
    let short_wall = Wall::standard(room_depth, wall_height);

    let half_l = room_length / 2.0;
    let half_d = room_depth / 2.0;

    let mut all_members = Vec::new();

    // Front wall (with window): along X at z = -half_d
    let front_framing = generate_wall_with_window(
        &long_wall,
        1.5,  // window starts 1.5m from left
        0.9,  // sill at 900mm
        1.2,  // 1200mm wide
        1.2,  // 1200mm tall
    );
    for mut m in front_framing.members {
        m.x -= half_l;
        m.z = -half_d;
        m.name = format!("Front {}", m.name);
        all_members.push(m);
    }

    // Back wall: along X at z = +half_d
    for mut m in long_wall.generate_framing().members {
        m.x -= half_l;
        m.z = half_d;
        m.name = format!("Back {}", m.name);
        all_members.push(m);
    }

    // Left wall: along Z at x = -half_l (rotate: wall x -> world z)
    for mut m in short_wall.generate_framing().members {
        let local_x = m.x;
        m.x = -half_l;
        m.z = local_x - half_d;
        // Swap width/depth for 90° rotation
        let w = m.width;
        m.width = m.depth;
        m.depth = w;
        m.name = format!("Left {}", m.name);
        all_members.push(m);
    }

    // Right wall: along Z at x = +half_l
    for mut m in short_wall.generate_framing().members {
        let local_x = m.x;
        m.x = half_l;
        m.z = local_x - half_d;
        let w = m.width;
        m.width = m.depth;
        m.depth = w;
        m.name = format!("Right {}", m.name);
        all_members.push(m);
    }

    // Concrete slab: 100mm thick, flush with ground, slightly overhangs walls
    let slab_thickness = 0.100;
    let slab_overhang = 0.150; // 150mm overhang each side
    all_members.push(FramingMember {
        name: "Concrete Slab".into(),
        member_type: "slab".into(),
        x: 0.0,
        y: -(slab_thickness / 2.0),
        z: 0.0,
        width: room_length + slab_overhang * 2.0,
        height: slab_thickness,
        depth: room_depth + slab_overhang * 2.0,
        color: 0xBBBBBB, // concrete grey
        rotation_axis: None,
    });

    // Plumbing: cold water pipe running through back wall studs at 500mm height
    let pipe_diameter = 0.020; // 20mm copper pipe
    let pipe_y = 0.500;
    // Horizontal run along the back wall
    all_members.push(FramingMember {
        name: "Cold Water Pipe".into(),
        member_type: "pipe".into(),
        x: 0.0,
        y: pipe_y,
        z: half_d,
        width: room_length * 0.7, // runs ~70% of wall length
        height: pipe_diameter,
        depth: pipe_diameter,
        color: 0x3399FF, // blue = cold water
        rotation_axis: Some(0), // along X axis
    });
    // Vertical riser from floor up to the horizontal pipe
    all_members.push(FramingMember {
        name: "Cold Water Riser".into(),
        member_type: "pipe".into(),
        x: -(room_length * 0.35),
        y: pipe_y / 2.0,
        z: half_d,
        width: pipe_y, // length = height from floor to pipe
        height: pipe_diameter,
        depth: pipe_diameter,
        color: 0x3399FF,
        rotation_axis: None, // vertical (default cylinder orientation)
    });
    // Hot water pipe slightly above cold
    all_members.push(FramingMember {
        name: "Hot Water Pipe".into(),
        member_type: "pipe".into(),
        x: 0.0,
        y: pipe_y + 0.080, // 80mm above cold
        z: half_d,
        width: room_length * 0.5,
        height: pipe_diameter,
        depth: pipe_diameter,
        color: 0xFF4444, // red = hot water
        rotation_axis: Some(0),
    });
    // Hot water riser
    all_members.push(FramingMember {
        name: "Hot Water Riser".into(),
        member_type: "pipe".into(),
        x: -(room_length * 0.25),
        y: (pipe_y + 0.080) / 2.0,
        z: half_d,
        width: pipe_y + 0.080,
        height: pipe_diameter,
        depth: pipe_diameter,
        color: 0xFF4444,
        rotation_axis: None,
    });

    let data = WallViewerData { members: all_members };
    serde_json::to_string(&data).unwrap_or_default()
}
