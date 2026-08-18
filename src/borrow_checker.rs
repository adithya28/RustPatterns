use std::marker::PhantomPinned;
use std::ptr::NonNull;

struct BorrowChecker {
    data: i64,
}
impl BorrowChecker {
    fn new(val: i64) -> Self {

        let item = BorrowChecker { data: val };
        item


    }
}

fn main()
{
    let mut checker = BorrowChecker
    {
        data: 0,
    };
    //this violates borrow checker's rule that states there's only one mutable ref and infinite immutable refs.
    let mut_ref= &mut checker;
    let immut_ref = &checker;
    mut_ref.data += 2;

    // this is fine

    let mut_ref = &mut checker;
    let immut_ref = &checker;
    let data :i32 = immut_ref.data as i32;
}
