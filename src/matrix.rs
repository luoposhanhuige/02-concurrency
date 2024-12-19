use anyhow::{anyhow, Result}; // anyhow::anyhow 是个宏，用来创建一个 anyhow::Error 类型的错误。Result 是一个类型别名，它是 anyhow::Result 类型的别名。
use std::{
    fmt,
    ops::{Add, AddAssign, Mul},
};

// 声明一个矩阵的结构
// [[1, 2], [1, 2], [1, 2]] => [1, 2, 1, 2, 1, 2] // 计算机比较喜欢后一种形式，因为它更加紧凑。前一种形式中，每个元素都是一个数组，指针指向增加复杂性
// #[derive(Debug)]
// pub struct Matrix<T: Debug> { // T: Debug 表示 T 必须实现 Debug trait，这样，我们就可以用 {:?} 来输出 T 类型的实例。
//     data: Vec<T>, // 一维数组，其中包含矩阵的所有元素。其中 T 用泛型表示，可以是任意类型。如果用 i32 表示，那么这个矩阵就是一个整数矩阵。
//     row: usize,
//     col: usize,
// }

pub struct Matrix<T> {
    data: Vec<T>, // 一维数组，其中包含矩阵的所有元素。其中 T 用泛型表示，可以是任意类型。如果用 i32 表示，那么这个矩阵就是一个整数矩阵。
    row: usize,
    col: usize,
}

pub fn multiply<T>(a: &Matrix<T>, b: &Matrix<T>) -> Result<Matrix<T>>
where
    T: Mul<Output = T> + Add<Output = T> + AddAssign + Default + Copy,
{
    // + Debug
    if a.col != b.row {
        return Err(anyhow!("Matrix dimensions do not match, a.col != b.row"));
    }
    // let mut data = vec![0; a.row * b.col];
    // let mut data = Vec::with_capacity(a.row * b.col);
    let mut data = vec![T::default(); a.row * b.col];
    for i in 0..a.row {
        for j in 0..b.col {
            for k in 0..a.col {
                data[i * b.col + j] += a.data[i * a.col + k] * b.data[k * b.col + j];
                // +=, AddAssign trait  // copy trait 保证了 T 类型的实例可以直接赋值给 data[i * b.col + j]。
            }
        }
    }
    // 矩阵相乘的结果，是一个 a.row * b.col 的矩阵，这个矩阵中的每个元素的下标是 i * b.col + j，其中 i 是行号，j 是列号。
    // 上述的计算可以这么思考：从最终结果的矩阵中，定位任意一个元素为：data[i * b.col + j]；然后，这个元素是由 a 矩阵的第 i 行和 b 矩阵的第 j 列相乘得到的。
    // 所以，我们需要遍历 a 矩阵的第 i 行和 b 矩阵的第 j 列，然后，把它们的乘积累加到 data[i * b.col + j] 中。
    // k 是 a 矩阵的列号，也是 b 矩阵的行号。
    Ok(Matrix {
        data,
        row: a.row,
        col: b.col,
    })
}

// why we need to implement Debug trait?
// Because we want to print the matrix in a debug format.
// what is debug format?
// Debug format is a format that is used to print the data in a way that is easy to debug.
impl<T: fmt::Debug> Matrix<T> {
    pub fn new(data: impl Into<Vec<T>>, row: usize, col: usize) -> Self {
        Self {
            data: data.into(),
            row,
            col,
        }
    }
}

// why we need to implement Display trait?
// Because we want to print the matrix in a human-readable format.

impl<T> fmt::Display for Matrix<T>
where
    T: fmt::Display,
{
    // display a 2x3 as {1 2 3, 4 5 6}, 3x2 as {1 2, 3 4, 5 6}
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{")?;
        for i in 0..self.row {
            for j in 0..self.col {
                write!(f, "{}", self.data[i * self.col + j])?;
                if j != self.col - 1 {
                    write!(f, " ")?;
                }
            }

            if i != self.row - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "}}")?;
        Ok(())
    }
}

impl<T> fmt::Debug for Matrix<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Matrix(row={}, col={}, {})", self.row, self.col, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*; // use super::*; 表示引入当前模块的父模块中的所有内容。super 表示父模块，* 表示所有内容。

    #[test]
    fn test_matrix_multiply() -> Result<()> {
        let a = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new([1, 2, 3, 4, 5, 6], 3, 2);
        let c = multiply(&a, &b)?;
        assert_eq!(c.col, 2);
        assert_eq!(c.row, 2);
        assert_eq!(c.data, vec![22, 28, 49, 64]);
        assert_eq!(format!("{:?}", c), "Matrix(row=2, col=2, {22 28, 49 64})");

        Ok(())
    }

    #[test]
    fn test_matrix_display() -> Result<()> {
        let a = Matrix::new([1, 2, 3, 4], 2, 2);
        let b = Matrix::new([1, 2, 3, 4], 2, 2);
        let c = multiply(&a, &b)?;
        assert_eq!(c.data, vec![7, 10, 15, 22]);
        assert_eq!(format!("{}", c), "{7 10, 15 22}");
        Ok(())
    }
}
