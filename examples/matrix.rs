use anyhow::Result;
use concurrency::Matrix;

fn main() -> Result<()> {
    // println!("i32: default: {:?}", i32::default());
    let a = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
    let b = Matrix::new([1, 2, 3, 4, 5, 6], 3, 2);
    //println!("a * b: {:?}", a * b); //a * b: Matrix(row=2, col=2, {22 28, 49 64})
    println!("a * b: {}", a * b); //a * b: {22 28, 49 64}
    Ok(())
}
