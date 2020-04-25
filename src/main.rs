extern crate num_bigint;
extern crate num_traits;
extern crate modinverse;

use std::mem::replace;

use num_bigint::{BigUInt, ToBigInt, BigInt, Sign};
use num_traits::{Zero, One};

use modinverse::modinverse;

struct Primitives {
    width : usize,
    m : usize,
    n : BigInt,
    r : BigInt,
    inv : BigInt,
    exp : BigInt,
    dexp : BigInt
}

fn make_primitives(width : usize) -> Primitives {
    let exp = ToBigInt::to_bigint(&65537);

    // when picking p and q ensure that exp is coprime to (p - 1) and (q - 1)
    let p = BigInt::parse_bytes(b"148715450009406215544851404632970187055827987065657638482806886212433040335306709375657187526428825846441364529015687294103246278898356371822895605129223904973676488684092873351933448486276982448871026358660306862802318591414961154419903429418827756491839789904016335301529951459787672947843136480252146828697", 10);
    let q = BigInt::parse_bytes(b"50729716869762510472957508551392924763015740203992845666542245466801494776486627862032821248558918608127925860412000975488095246401743393430586416646520115373885609513042364141025003284261617251855726831653550557210202048890913945850247146994086207936675898070735266171909273984651769282447133701615228003941", 10);

    let n = p * q;

    let dexp : BigInt = match modinverse(exp, (p - 1)*(q - 1)) {
        Some v0 => v0,
        None => panic!("couldnt get dexp"),
    }

    // width must be a power of 2
    // n will be odd.
    // r is 2^(2*width)
    // n will be less than r.

    let prims = Primitives {
        width: width,
        m : width * 2,
        n : n,
        r : make_r(m),
        inv : make_inv(m, n),
        exp : exp,
        dexp : dexp,
    }


}



fn make_r(m : usize) -> BigInt {
    let power = m / 32;

    // to make r, we need to represent it as a base 2^32 number to pass
    // it to the BigInt constructor
    let digits = Vec::<u32>::new();
    for _ in 0..power {
        digits.push(0);
    }
    let leftoverexp = m - 32 * power;
    digits.push(2^leftoverexp);
    BigInt::new(Sign::Plus, digits);
}

fn make_inv(m : usize, n : BigInt) -> BigInt{
    let mut inv : BigInt = 1;
    let e = m.trailing_zeros() + 1;
    for _ in 1..e {
        inv = inv * (2 - n * inv);
    }
    inv;
}


// compute x * r^(-1) mod n
// r = 2^m
// The parameters satisify r * r^(-1) + n * inv = 1

fn reduce(x : BigInt) {
    let xmodr : BigInt = x && (r - 1);
    let inv_modr : BigInt = inv && (r - 1);
    let q = xmodr * inv_modr;
    let a = (x - q * n) >> m;
    if a < 0 {
        a = a + n;
    }
    a
}

// assumes the multiplication is being done modulo n and that the r value
// to put the numbers in montgomery space is r.
fn mont_mult(x : BigInt, y : BigInt) -> BigInt{
    reduce(x * y);
}

// mess must be smaller than n
fn mod_exp_by_squaring(mess : BigInt, exp : BigInt, n : BigInt) -> BigInt {
    let mut curr_exp = 1;
    let mut res = mess;

    // TODO maybe this loop could be faster by calculating log first and then
    // doing additions instead of doing some multiplications
    while (curr_exp  * 2 <= exp) {
        res = mont_mult(res, res);
        curr_exp = curr_exp * 2;
    }
    res = res * mod_exp_by_squaring(mess, exp - curr_exp, n);
    res
}


fn encrypt(mess : BigInt, exp : BigInt, n : BigInt) -> BigInt {
    mod_exp_by_squaring(mess, exp, n);
}

fn decrypt(ciphertext : BigInt, dexp : BigInt, n : BigInt) -> BigInt {
    mod_exp_by_squaring(ciphertext, dexp, n);
}


// Calculate large fibonacci numbers.
fn fib(n: usize) -> BigUint {
    let mut f0: BigUint = Zero::zero();
    let mut f1: BigUint = One::one();
    for _ in 0..n {
        let f2 = f0 + &f1;
        // This is a low cost way of swapping f0 with f1 and f1 with f2.
        f0 = replace(&mut f1, f2);
    }
    f0
}


fn main() {
    




    println!("fib(1000) = {}", fib(1000));
}
