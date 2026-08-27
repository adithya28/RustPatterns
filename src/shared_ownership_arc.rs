/* ARC is a smart pointer, used to share ownership of a value across threads and multiple variables.
Ideally used when multiple threads need access to the same data, it works by copying references in the
programming heap and keeps operations strictly atomic.
*/

use std::sync::Arc;
#[derive(Debug)]
struct CustomType
{
    data:i64
}
fn main()
{
    let custom_var = Arc::new(CustomType {data:42}); // using ARC to initialize it on the HEAP as an atomic op.
    let custom_var_clone = custom_var.clone();
    println!("custom var is {}", custom_var_clone.data);
    println!("custom var is {:?}", custom_var);
}

