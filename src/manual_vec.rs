/*
fn make_zero(bits : i32) -> Vec<u8> {
    let mut v = Vec::new();
    let bytes = bits / 8;
    for i in 0..bytes {
        v.push(0);
    }
    v
}
*/

/*
fn make_rand(bits : i32) -> Vec<u8> {
    let mut v = Vec::new();
    let bytes = bits / 8;
    for i in 0..bytes {
        v.push(random());
    }
    v
}

*/

fn mult_u8(x : u8, y : u8) -> Vec<u8> {
    let res = (x as u16) * (y as u16);

    let mut v = Vec::new();
    v.push(res as u8);
    v.push((res >> 8) as u8);
    v
}

fn add_u8(x : u8, y : u8) -> Vec<u8> {
    let res = (x as u16) + (y as u16);

    let mut v = Vec::new();
    v.push(res as u8);
    v.push((res >> 8) as u8);
    v
}

/*
fn add_vecu8_u8(&mut vx : Vec<u8>, y : u8) {
    let mut low_bits = y;
    for i in 0..(vx.len()) {
        let sum = (vx[i] as u16) + (low_bits as u16);
        vx[i] = sum as u8;
        low_bits = (sum >> 8) as u8;
        if low_bits == 0 {
            return;
        }
    }
    if low_bits != 0 {
        vx.push(low_bits);
    }
}
*/

fn add_vecu8(vx : &Vec<u8>, vy : &Vec<u8>) -> Vec<u8> {
    let longv;
    let shortv;

    if vx.len() < vy.len() {
        longv = vy;
        shortv = vx;
    } else {
        longv = vx;
        shortv = vy;
    }

    let mut vz = Vec::<u8>::new();
    let mut low_bits : u16 = 0;
    for i in 0..shortv.len() {
        let sum = (shortv[i] as u16) + (longv[i] as u16) + low_bits;
        vz.push(sum as u8);
        low_bits = sum >> 8;
    }
    for i in shortv.len()..longv.len() {
        let sum = (longv[i] as u16) + low_bits;
        vz.push(sum as u8);
        low_bits = sum >> 8;
    }
    if low_bits != 0 {
        vz.push(low_bits as u8);
    }
    vz
}

fn vecu8_u64(v : &Vec<u8>) -> u64 {
    let mut pow = 1;
    let mut res = 0;
    for &c in v {
        res = res + (c as u64) * pow;
        pow = pow * 256;
    }
    res
}

fn u64_vecu8(n : u64) -> Vec<u8> {
    let mut v = Vec::new();
    let mut m = n;
    if n == 0 {
        v.push(0);
        return v;
    }
    while m != 0 {
        v.push(m as u8);
        m = m >> 8;
    }
    v
}

#[test]
fn add_test() {
    assert_eq!(13 + 38, vecu8_u64(&add_vecu8(&u64_vecu8(13), &u64_vecu8(38))));

    let n : u64 = 987602;
    let m : u64 = 7038819011183;
    let v1 = u64_vecu8(n);
    let v2 = u64_vecu8(m);
    let v3 = add_vecu8(&v1, &v2);
    assert_eq!(n + m, vecu8_u64(&v3));
}


fn main() {
    println!("hello");
}

