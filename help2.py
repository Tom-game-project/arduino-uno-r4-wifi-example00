"""
rust code 
// Define pin bits for LED matrix control
// P0xx pins
const P003_BIT: u16 = 1 << 3;
const P004_BIT: u16 = 1 << 4;
const P011_BIT: u16 = 1 << 11;
const P012_BIT: u16 = 1 << 12;
const P013_BIT: u16 = 1 << 13;
const P015_BIT: u16 = 1 << 15;

// P2xx pins
const P204_BIT: u16 = 1 << 4;
const P205_BIT: u16 = 1 << 5;
const P206_BIT: u16 = 1 << 6;
const P212_BIT: u16 = 1 << 12;
const P213_BIT: u16 = 1 << 13;
"""

rows = [
    "ROW0" ,
    "ROW1" ,
    "ROW2" ,
    "ROW3" ,
    "ROW4" ,
    "ROW5" ,
    "ROW6" , 
    "ROW7" , 
    "ROW8" , 
    "ROW9" , 
    "ROW10"
]


rows_codes = [
    "P003_BIT",
    "P004_BIT",
    "P011_BIT",
    "P012_BIT",
    "P013_BIT",
    "P015_BIT",
    "P204_BIT",
    "P205_BIT",
    "P206_BIT",
    "P212_BIT",
    "P213_BIT",
]

d = {
# P0xx pins
    "ROW0" : "P003_BIT",
    "ROW1" : "P004_BIT",
    "ROW2" : "P011_BIT",
    "ROW3" : "P012_BIT",
    "ROW4" : "P013_BIT",
    "ROW5" : "P015_BIT",
# P2xx pins
    "ROW6": "P204_BIT",
    "ROW7": "P205_BIT",
    "ROW8": "P206_BIT",
    "ROW9": "P212_BIT",
    "ROW10":"P213_BIT",
}

def gen_code_segment(high:str ,low:str) -> str:
    return """
    device.PORT0.pdr().write(|w| unsafe { w.pdr().bits(%s | %s) }); // Set both as outputs
    device.PORT0.podr().write(|w| unsafe { w.podr().bits(%s) }); // P003 High, P004 Low
    """ % (low, high, high)


