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

fn add_vecu8(vx : Vec<u8>, vy : Vec<u8>) -> Vec<u8> {
    let mut longv = vx;
    let mut shortv = vy;
    if vx.len() < vy.len() {
        longv = vy;
        shortv = vx;
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
    vz
}


fn main() {
    let v1 = Vec::<u8>::new();
    v1.push(10);
    v1.push(10);
    let v2 = Vec::<u8>::new();
    v2.push(10);
    v2.push(10);
    let v3 = add_vecu8(v1, v2);
    println!("v1 {:?}\nv2 {:?}\nv3 {:?}", v1, v2, v3);
}

