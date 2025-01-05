use anyhow::{anyhow, Result}; // anyhow::anyhow 是个宏，用来创建一个 anyhow::Error 类型的错误。Result 是一个类型别名，它是 anyhow::Result 类型的别名。
use std::{
    fmt,
    ops::{Add, AddAssign, Mul},
    sync::mpsc,
    thread,
};

use crate::{dot_product, Vector};
// what is crate?
// crate 是一个 Rust 项目的根目录。在一个 crate 中，可以有多个模块，每个模块可以包含多个函数、结构体、枚举等。

const NUM_THREADS: usize = 4;

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
    T: Mul<Output = T> + Add<Output = T> + AddAssign + Default + Copy + Send + 'static,
{
    // + Debug
    if a.col != b.row {
        return Err(anyhow!("Matrix dimensions do not match, a.col != b.row"));
    }

    // (0..NUM_THREADS) is a iteraor. map is a iterator adapter.
    let senders = (0..NUM_THREADS)
        .map(|_| {
            // 不需要返回值。创建 NUM_THREADS 个线程，每个线程都有一个 mpsc::Sender 实例。
            let (tx, rx) = mpsc::channel::<Msg<T>>(); //channel 的泛型参数，需要把要传递的数据类型传给它。

            thread::spawn(move || {
                for msg in rx {
                    let value = dot_product(msg.input.row, msg.input.col)?;
                    // 做完 dot_product 之后，把结果发送给发送者。
                    // 2, 因为 error 不能在两个线程中发送，所以这里需要用 if let Err(e) = msg.sender.send(MsgOutput { ... }) {} 来处理错误。
                    if let Err(e) = msg.sender.send(MsgOutput {
                        idx: msg.input.idx,
                        value,
                    }) {
                        eprintln!("Send error: {:?}", e);
                    }
                }
                Ok::<_, anyhow::Error>(()) // 1，因为编译器需要确定错误的类型，所以这里需要 Ok::<_, anyhow::Error>(())。
            });
            tx
        })
        .collect::<Vec<_>>();

    // let mut data = vec![0; a.row * b.col];
    // let mut data = Vec::with_capacity(a.row * b.col);
    let matrix_len = a.row * b.col;
    let mut data = vec![T::default(); matrix_len];
    let mut receivers = Vec::with_capacity(matrix_len);

    // for i in 0..a.row {
    //     for j in 0..b.col {
    //         for k in 0..a.col {
    //             data[i * b.col + j] += a.data[i * a.col + k] * b.data[k * b.col + j]; // 因为每个Matrix<T>的data是Vec<T>，所以可以直接用[]访问
    //             // +=, AddAssign trait  // copy trait 保证了 T 类型的实例可以直接赋值给 data[i * b.col + j]。
    //         }
    //     }
    // }
    // map-reduce: map phrase
    for i in 0..a.row {
        for j in 0..b.col {
            // 如果把 下面一行，移到 上一层循环中，dot_product(row, col)? 会报错，因为 used of moved value: row
            // 除非把 dot_product(row.clone(), col)?，但是这样会导致性能下降。
            // 或者把引用传给 dot_product(&row, &col)?，但是这样会导致 dot_product 的签名变得复杂。需要把 dot_product 的签名改为 fn dot_product<T>(a: &Vector<T>, b: &Vector<T>) -> Result<T>，而且 Vector<T> 也要修改。
            // 所以，这里，我们选择在内层循环中，创建 row 和 col 的实例。
            let row = Vector::new(&a.data[i * a.col..(i + 1) * a.col]); // creates a new Vector instance from the slice representing the i-th row of the matrix a.
            let col_data = b.data[j..]
                .iter()
                .step_by(b.col)
                .copied()
                .collect::<Vec<_>>();
            let col = Vector::new(col_data);
            let idx = i * b.col + j;
            let input = MsgInput::new(idx, row, col);
            let (tx, rx) = oneshot::channel();
            let msg = Msg::new(input, tx);
            if let Err(e) = senders[idx % NUM_THREADS].send(msg) {
                // sender 是一个 vec of tx（其实就是mpsc::Sender<Msg<T>>），所以，我们可以用 idx % NUM_THREADS 来选择一个发送者。
                eprintln!("Send error: {:?}", e);
            }

            receivers.push(rx);
        }
    }

    // map-reduce: reduce phrase
    for rx in receivers {
        let MsgOutput { idx, value } = rx.recv()?;
        data[idx] = value;

        // let output= rx.recv()?;
        // data[output.idx] = output.value;
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

pub struct MsgInput<T> {
    idx: usize,
    row: Vector<T>,
    col: Vector<T>,
}

pub struct MsgOutput<T> {
    idx: usize,
    value: T, // why not use Vector<T>? Because the result of dot_product is a scalar, not a vector. // what is scalar? // scalar 是一个数，而不是一个向量。
}

pub struct Msg<T> {
    input: MsgInput<T>,
    sender: oneshot::Sender<MsgOutput<T>>, // tx
}

// 为 Matrix<T> 实现 Mul trait，这样，我们就可以通过 * 运算符，来实现矩阵相乘。
// 查看下面测试用例 test_matrix_multiply()，可以看到，通过 a * b，就可以实现矩阵相乘。
impl<T> Mul for Matrix<T>
where
    T: Mul<Output = T> + Add<Output = T> + AddAssign + Default + Copy + Send + 'static,
{
    type Output = Self; //Matrix<T>

    fn mul(self, rhs: Self) -> Self::Output {
        multiply(&self, &rhs).expect("Matrix multiply error!") // 核心语句，这样通过 * 运算符，间接调用 multiply，就可以实现相乘。
    }
}

// why we need to implement Debug trait?
// Because we want to print the matrix in a debug format.
// what is debug format?
// Debug format is a format that is used to print the data in a way that is easy to debug.
// This snippet defines a constructor for the Matrix struct and requires that T implements the Debug trait.
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

impl<T> MsgInput<T> {
    pub fn new(idx: usize, row: Vector<T>, col: Vector<T>) -> Self {
        Self { idx, row, col }
    }
}

impl<T> Msg<T> {
    pub fn new(input: MsgInput<T>, sender: oneshot::Sender<MsgOutput<T>>) -> Self {
        Self { input, sender }
    }
}

#[cfg(test)]
mod tests {
    use super::*; // use super::*; 表示引入当前模块的父模块中的所有内容。super 表示父模块，* 表示所有内容。

    #[test]
    fn test_matrix_multiply() -> Result<()> {
        let a = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new([1, 2, 3, 4, 5, 6], 3, 2);
        let c = a * b; //multiply(&a, &b)?;
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
        let c = a * b; //multiply(&a, &b)?;
        assert_eq!(c.data, vec![7, 10, 15, 22]);
        assert_eq!(format!("{}", c), "{7 10, 15 22}");
        Ok(())
    }

    #[test]
    fn test_a_can_not_multiply_b() {
        let a = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new([1, 2, 3, 4], 2, 2);
        let c = multiply(&a, &b);
        assert!(c.is_err()); // assert!(c.is_err()); 表示 c 是一个错误。
    }

    #[test]
    #[should_panic]
    fn test_a_can_not_multiply_b_panic() {
        let a = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new([1, 2, 3, 4], 2, 2);
        let _c = a * b; //_c 直接丢弃，不需要用 assert!() 来判断是否成功。
                        // assert!(c.is_err());
                        // Ok(())
    }
}
