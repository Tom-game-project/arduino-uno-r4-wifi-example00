import math

pin_mapping = {
    "ROW0" : "P205_BIT",
    "ROW1" : "P012_BIT",
    "ROW2" : "P013_BIT",
    "ROW3" : "P206_BIT",
    "ROW4" : "P003_BIT",
    "ROW5" : "P204_BIT",
    "ROW6": "P015_BIT",
    "ROW7": "P004_BIT",
    "ROW8": "P011_BIT",
    "ROW9": "P213_BIT",
    "ROW10":"P212_BIT",
}


def r(n:int) -> int:
    return math.floor((math.sqrt(4 * n - 3) - 1) / 2)


def c(n):
    rv = r(n)
    return n - (rv*(rv + 1) + 1)


def row_top(n:int) -> str:
    return f"ROW{r(n) + 1}"


def row_bot(n:int) -> str:
    return f"ROW{c(n)//2}"


def gen_tup_str(n:int) -> str:
    if c(n) % 2== 0:
        return "(%s, %s)," % (pin_mapping[row_top(n)], pin_mapping[row_bot(n)])
    else:
        return "(%s, %s)," % (pin_mapping[row_bot(n)], pin_mapping[row_top(n)],)


def main():
    for n in range(1,97):
        print(gen_tup_str(n))



if __name__ == "__main__":
    main()
