struct CustomType
{
    data:i64
}

fn main()
{
    let mut customVar = CustomType{data:42};
    changeData(&mut customVar);
    printData(&customVar);

}
fn changeData(input_var: &mut CustomType)
{
    input_var.data = 200;
}
fn printData(input_var: &CustomType)
{
    println!("Data: {}", input_var.data);
}