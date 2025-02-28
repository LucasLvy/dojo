use serde::Serde;
use array::ArrayTrait;

#[derive(Drop)]
struct ToCallData {
    data: Array<felt252>,
}

#[generate_trait]
impl ToCallDataImpl of ToCallDataTrait {
    fn plus<T, impl TSerde: Serde<T>, impl TD: Drop<T>>(
        mut self: ToCallData, data: T
    ) -> ToCallData {
        data.serialize(ref self.data);
        self
    }
}

fn to_calldata<T, impl TSerde: Serde<T>, impl TD: Drop<T>>(data: T) -> ToCallData {
    let mut calldata: Array<felt252> = ArrayTrait::new();
    data.serialize(ref calldata);
    ToCallData { data: calldata }
}


fn system_calldata<T, impl TSerde: Serde<T>, impl TD: Drop<T>>(data: T) -> Array<felt252> {
    let mut calldata: Array<felt252> = ArrayTrait::new();
    data.serialize(ref calldata);
    calldata
}


impl PartialEqArray<T, impl TPEq: PartialEq<T>> of PartialEq<Array<T>> {
    fn eq(lhs: @Array<T>, rhs: @Array<T>) -> bool {
        if lhs.len() != rhs.len() {
            return false;
        };

        let mut is_eq = true;
        let mut i = 0;
        loop {
            if lhs.len() == i {
                break;
            };
            if lhs.at(i) != rhs.at(i) {
                is_eq = false;
                break;
            };

            i += 1;
        };

        is_eq
    }

    fn ne(lhs: @Array<T>, rhs: @Array<T>) -> bool {
        !PartialEqArray::eq(lhs, rhs)
    }
}
