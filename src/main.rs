
// reduce x
// q is x * n_prime modulus r where r is 2^m
// the goal is to compute x * r^(-1) mod n, and we want to be able to represent
// all numbers between 0 and n-1 anyway, so n can't be larger than what our
// chosen datatype can represent.

// So if all of these numbers are being represented with a u8, then n is
// limited at 256. Since n is limited at 256, and the return value is mod n,
// then the return value can be represented by a u8.

fn reduce(x : u8, n : u8, n_prime : u8, m : u8, r : u8) -> u8 {
    let xmodr = (x && (r - 1));
    let n_prime_modr = (n_prime_modr && (r - 1));
    let q : u16 = (xmodr as u16) * (n_prime_modr as u16);
    let qmodr = (q && (r - 1)):
    let q2 : u16 = (qmodr as u16) * (n as u16);
    let a : i32 = ((x as i32) - (q2 as i32)) >> m;
    if a < 0 {
        a = a + n;
    }
    (a as u8)
}







fn main() {
    println!("hello");
}

