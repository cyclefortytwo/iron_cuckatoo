extern crate iron_cuckatoo;
extern crate ocl;

use iron_cuckatoo::{Graph, Trimmer};
use std::env;
use std::time::SystemTime;

fn main() -> ocl::Result<()> {
    let args: Vec<String> = env::args().collect();
    let start = SystemTime::now();

    let platform_selector = if args.len() >= 2 {
        Some(&args[1][..])
    } else {
        None
    };

    let device_selector = if args.len() >= 3 {
        if let Ok(v) = &args[2].parse::<usize>() {
            Some(*v)
        } else {
            return Err("Device ID must be a number".into());
        }
    } else {
        None
    };

    let trimmer = Trimmer::build(platform_selector, device_selector, 29)?;
    for i in 0..2 {
        let k = if i == 0 {
            [
                0x27580576fe290177,
                0xf9ea9b2031f4e76e,
                0x1663308c8607868f,
                0xb88839b0fa180d0e,
            ]
        } else {
            [
                0x5c0348cfc71b5ce6,
                0xbf4141b92a45e49,
                0x7282d7893f658b88,
                0x61525294db9b617f,
            ]
        };

        let res = trimmer.run(&k)?;
        println!("Trimmed to {}", res.len());

        let m3 = SystemTime::now();
        let g = Graph::build(&res);
        let m4 = SystemTime::now();
        println!("Building graph {:?}", m4.duration_since(m3).unwrap());
        println!(
            "Number of nodes {}, edges {}",
            g.node_count(),
            g.edge_count()
        );

        let m5 = SystemTime::now();
        let solutions = g.find()?;
        let m6 = SystemTime::now();
        println!("Searching graph {:?}", m6.duration_since(m5).unwrap());
        println!(
            "Total time in cycle finder {:?}",
            m6.duration_since(m3).unwrap()
        );
        for sol in solutions {
            println!("Solution: {:x?}", sol.nonces);
        }
    }

    Ok(())
}
