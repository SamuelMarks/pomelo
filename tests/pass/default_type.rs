use pomelo::*;

pomelo! {
    %default_type Option<i32>;

    input ::= Terminal(T) { let x : Option<i32> = T; x }
}
