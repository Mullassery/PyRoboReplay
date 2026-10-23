#[path = "../tests/support/mod.rs"]
mod support;

fn main() -> anyhow::Result<()> {
    let output_path = "warehouse_exploration_v1.db3";
    support::generate_bag(output_path)?;

    println!("\n📊 Warehouse Mission Generated");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Location: ./{output_path}");
    println!("\n🎬 Try replaying:");
    println!("   cargo run --release -- replay {output_path}");
    println!("\n📡 List sensors:");
    println!("   cargo run --release -- list {output_path}");
    println!("\n📊 Analyze mission:");
    println!("   cargo run --release -- analyze {output_path}");

    Ok(())
}
