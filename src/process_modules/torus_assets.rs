pub fn ensure_torus_assets(major_r: f32, minor_r: f32) -> String {
    let model_name = format!(
        "torus_R{}_r{}",
        (major_r * 1000.0).round() as i32,
        (minor_r * 1000.0).round() as i32
    );
    let obj_path = format!("assets/{}.obj", model_name);
    let compound_path = format!("assets/{}.compound", model_name);
    if !std::path::Path::new(&obj_path).exists() {
        write_torus_obj(&obj_path, major_r, minor_r);
        println!("  ↳ generado {}", obj_path);
    }
    if !std::path::Path::new(&compound_path).exists() {
        rapier_bevy::preprocess_obj(
            &obj_path,
            &compound_path,
            None,
            rapier_bevy::VHACDParameters {
                resolution: 64,
                ..Default::default()
            },
        );
    }
    model_name
}

fn write_torus_obj(path: &str, major_r: f32, minor_r: f32) {
    let n_major = 48usize;
    let n_minor = 24usize;
    let mut s = String::new();
    s.push_str("# torus generado por process_modules\n");
    for i in 0..n_major {
        let phi = std::f32::consts::TAU * (i as f32) / (n_major as f32);
        let (sin_phi, cos_phi) = phi.sin_cos();
        for j in 0..n_minor {
            let theta = std::f32::consts::TAU * (j as f32) / (n_minor as f32);
            let (sin_theta, cos_theta) = theta.sin_cos();
            let x = (major_r + minor_r * cos_theta) * cos_phi;
            let y = (major_r + minor_r * cos_theta) * sin_phi;
            let z = minor_r * sin_theta;
            s.push_str(&format!("v {:.6} {:.6} {:.6}\n", x, y, z));
        }
    }
    for i in 0..n_major {
        let i_next = (i + 1) % n_major;
        for j in 0..n_minor {
            let j_next = (j + 1) % n_minor;
            let a = i * n_minor + j + 1;
            let b = i_next * n_minor + j + 1;
            let c = i_next * n_minor + j_next + 1;
            let d = i * n_minor + j_next + 1;
            s.push_str(&format!("f {} {} {}\n", a, b, c));
            s.push_str(&format!("f {} {} {}\n", a, c, d));
        }
    }
    std::fs::write(path, s).unwrap_or_else(|_| panic!("No se pudo escribir {}", path));
}
