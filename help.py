import re
import pprint
import json

# 解析対象のデータ
data = """
*   **LEDの座標**: (0, 0)
    *   **LEDの番号**: 1
    *   **接続されている2つのROWピン**: ROW0, ROW1
    *   **HIGHにするROWピン**: **ROW0**
    *   **LOWにするROWピン**: **ROW1**

*   **LEDの座標**: (1, 0)
    *   **LEDの番号**: 2
    *   **接続されている2つのROWピン**: ROW1, ROW0
    *   **HIGHにするROWピン**: **ROW1**
    *   **LOWにするROWピン**: **ROW0**

*   **LEDの座標**: (2, 0)
    *   **LEDの番号**: 3
    *   **接続されている2つのROWピン**: ROW0, ROW2
    *   **HIGHにするROWピン**: **ROW0**
    *   **LOWにするROWピン**: **ROW2**

*   **LEDの座標**: (3, 0)
    *   **LEDの番号**: 4
    *   **接続されている2つのROWピン**: ROW2, ROW0
    *   **HIGHにするROWピン**: **ROW2**
    *   **LOWにするROWピン**: **ROW0**

*   **LEDの座標**: (4, 0)
    *   **LEDの番号**: 5
    *   **接続されている2つのROWピン**: ROW1, ROW2
    *   **HIGHにするROWピン**: **ROW1**
    *   **LOWにするROWピン**: **ROW2**

*   **LEDの座標**: (5, 0)
    *   **LEDの番号**: 6
    *   **接続されている2つのROWピン**: ROW2, ROW1
    *   **HIGHにするROWピン**: **ROW2**
    *   **LOWにするROWピン**: **ROW1**

*   **LEDの座標**: (6, 0)
    *   **LEDの番号**: 7
    *   **接続されている2つのROWピン**: ROW0, ROW3
    *   **HIGHにするROWピン**: **ROW0**
    *   **LOWにするROWピン**: **ROW3**

*   **LEDの座標**: (7, 0)
    *   **LEDの番号**: 8
    *   **接続されている2つのROWピン**: ROW3, ROW0
    *   **HIGHにするROWピン**: **ROW3**
    *   **LOWにするROWピン**: **ROW0**

*   **LEDの座標**: (8, 0)
    *   **LEDの番号**: 9
    *   **接続されている2つのROWピン**: ROW1, ROW3
    *   **HIGHにするROWピン**: **ROW1**
    *   **LOWにするROWピン**: **ROW3**

*   **LEDの座標**: (9, 0)
    *   **LEDの番号**: 10
    *   **接続されている2つのROWピン**: ROW3, ROW1
    *   **HIGHにするROWピン**: **ROW3**
    *   **LOWにするROWピン**: **ROW1**

*   **LEDの座標**: (10, 0)
    *   **LEDの番号**: 11
    *   **接続されている2つのROWピン**: ROW2, ROW3
    *   **HIGHにするROWピン**: **ROW2**
    *   **LOWにするROWピン**: **ROW3**

*   **LEDの座標**: (11, 0)
    *   **LEDの番号**: 12
    *   **接続されている2つのROWピン**: ROW3, ROW2
    *   **HIGHにするROWピン**: **ROW3**
    *   **LOWにするROWピン**: **ROW2**

*   **LEDの座標**: (0, 1)
    *   **LEDの番号**: 13
    *   **接続されている2つのROWピン**: ROW1, ROW4
    *   **HIGHにするROWピン**: **ROW1**
    *   **LOWにするROWピン**: **ROW4**

*   **LEDの座標**: (1, 1)
    *   **LEDの番号**: 14
    *   **接続されている2つのROWピン**: ROW4, ROW1
    *   **HIGHにするROWピン**: **ROW4**
    *   **LOWにするROWピン**: **ROW1**

*   **LEDの座標**: (2, 1)
    *   **LEDの番号**: 15
    *   **接続されている2つのROWピン**: ROW2, ROW4
    *   **HIGHにするROWピン**: **ROW2**
    *   **LOWにするROWピン**: **ROW4**

*   **LEDの座標**: (3, 1)
    *   **LEDの番号**: 16
    *   **接続されている2つのROWピン**: ROW4, ROW2
    *   **HIGHにするROWピン**: **ROW4**
    *   **LOWにするROWピン**: **ROW2**

*   **LEDの座標**: (4, 1)
    *   **LEDの番号**: 17
    *   **接続されている2つのROWピン**: ROW3, ROW4
    *   **HIGHにするROWピン**: **ROW3**
    *   **LOWにするROWピン**: **ROW4**

*   **LEDの座標**: (5, 1)
    *   **LEDの番号**: 18
    *   **接続されている2つのROWピン**: ROW4, ROW3
    *   **HIGHにするROWピン**: **ROW4**
    *   **LOWにするROWピン**: **ROW3**

*   **LEDの座標**: (6, 1)
    *   **LEDの番号**: 19
    *   **接続されている2つのROWピン**: ROW0, ROW5
    *   **HIGHにするROWピン**: **ROW0**
    *   **LOWにするROWピン**: **ROW5**

*   **LEDの座標**: (7, 1)
    *   **LEDの番号**: 20
    *   **接続されている2つのROWピン**: ROW5, ROW0
    *   **HIGHにするROWピン**: **ROW5**
    *   **LOWにするROWピン**: **ROW0**

*   **LEDの座標**: (8, 1)
    *   **LEDの番号**: 21
    *   **接続されている2つのROWピン**: ROW1, ROW5
    *   **HIGHにするROWピン**: **ROW1**
    *   **LOWにするROWピン**: **ROW5**

*   **LEDの座標**: (9, 1)
    *   **LEDの番号**: 22
    *   **接続されている2つのROWピン**: ROW5, ROW1
    *   **HIGHにするROWピン**: **ROW5**
    *   **LOWにするROWピン**: **ROW1**

*   **LEDの座標**: (10, 1)
    *   **LEDの番号**: 23
    *   **接続されている2つのROWピン**: ROW2, ROW5
    *   **HIGHにするROWピン**: **ROW2**
    *   **LOWにするROWピン**: **ROW5**

*   **LEDの座標**: (11, 1)
    *   **LEDの番号**: 24
    *   **接続されている2つのROWピン**: ROW5, ROW2
    *   **HIGHにするROWピン**: **ROW5**
    *   **LOWにするROWピン**: **ROW2**

*   **LEDの座標**: (0, 2)
    *   **LEDの番号**: 25
    *   **接続されている2つのROWピン**: ROW0, ROW4
    *   **HIGHにするROWピン**: **ROW0**
    *   **LOWにするROWピン**: **ROW4**

*   **LEDの座標**: (1, 2)
    *   **LEDの番号**: 26
    *   **接続されている2つのROWピン**: ROW4, ROW0
    *   **HIGHにするROWピン**: **ROW4**
    *   **LOWにするROWピン**: **ROW0**

*   **LEDの座標**: (2, 2)
    *   **LEDの番号**: 27
    *   **接続されている2つのROWピン**: ROW1, ROW5
    *   **HIGHにするROWピン**: **ROW1**
    *   **LOWにするROWピン**: **ROW5**

*   **LEDの座標**: (3, 2)
    *   **LEDの番号**: 28
    *   **接続されている2つのROWピン**: ROW5, ROW1
    *   **HIGHにするROWピン**: **ROW5**
    *   **LOWにするROWピン**: **ROW1**

*   **LEDの座標**: (4, 2)
    *   **LEDの番号**: 29
    *   **接続されている2つのROWピン**: ROW2, ROW5
    *   **HIGHにするROWピン**: **ROW2**
    *   **LOWにするROWピン**: **ROW5**

*   **LEDの座標**: (5, 2)
    *   **LEDの番号**: 30
    *   **接続されている2つのROWピン**: ROW5, ROW2
    *   **HIGHにするROWピン**: **ROW5**
    *   **LOWにするROWピン**: **ROW2**

*   **LEDの座標**: (6, 2)
    *   **LEDの番号**: 31
    *   **接続されている2つのROWピン**: ROW3, ROW5
    *   **HIGHにするROWピン**: **ROW3**
    *   **LOWにするROWピン**: **ROW5**

*   **LEDの座標**: (7, 2)
    *   **LEDの番号**: 32
    *   **接続されている2つのROWピン**: ROW5, ROW3
    *   **HIGHにするROWピン**: **ROW5**
    *   **LOWにするROWピン**: **ROW3**

*   **LEDの座標**: (8, 2)
    *   **LEDの番号**: 33
    *   **接続されている2つのROWピン**: ROW4, ROW5
    *   **HIGHにするROWピン**: **ROW4**
    *   **LOWにするROWピン**: **ROW5**

*   **LEDの座標**: (9, 2)
    *   **LEDの番号**: 34
    *   **接続されている2つのROWピン**: ROW5, ROW4
    *   **HIGHにするROWピン**: **ROW5**
    *   **LOWにするROWピン**: **ROW4**

*   **LEDの座標**: (10, 2)
    *   **LEDの番号**: 35
    *   **接続されている2つのROWピン**: ROW0, ROW6
    *   **HIGHにするROWピン**: **ROW0**
    *   **LOWにするROWピン**: **ROW6**

*   **LEDの座標**: (11, 2)
    *   **LEDの番号**: 36
    *   **接続されている2つのROWピン**: ROW6, ROW0
    *   **HIGHにするROWピン**: **ROW6**
    *   **LOWにするROWピン**: **ROW0**

*   **LEDの座標**: (0, 3)
    *   **LEDの番号**: 37
    *   **接続されている2つのROWピン**: ROW4, ROW0
    *   **HIGHにするROWピン**: **ROW4**
    *   **LOWにするROWピン**: **ROW0**

*   **LEDの座標**: (1, 3)
    *   **LEDの番号**: 38
    *   **接続されている2つのROWピン**: ROW0, ROW4
    *   **HIGHにするROWピン**: **ROW0**
    *   **LOWにするROWピン**: **ROW4**

*   **LEDの座標**: (2, 3)
    *   **LEDの番号**: 39
    *   **接続されている2つのROWピン**: ROW5, ROW1
    *   **HIGHにするROWピン**: **ROW5**
    *   **LOWにするROWピン**: **ROW1**

*   **LEDの座標**: (3, 3)
    *   **LEDの番号**: 40
    *   **接続されている2つのROWピン**: ROW1, ROW5
    *   **HIGHにするROWピン**: **ROW1**
    *   **LOWにするROWピン**: **ROW5**

*   **LEDの座標**: (4, 3)
    *   **LEDの番号**: 41
    *   **接続されている2つのROWピン**: ROW3, ROW6
    *   **HIGHにするROWピン**: **ROW3**
    *   **LOWにするROWピン**: **ROW6**

*   **LEDの座標**: (5, 3)
    *   **LEDの番号**: 42
    *   **接続されている2つのROWピン**: ROW6, ROW3
    *   **HIGHにするROWピン**: **ROW6**
    *   **LOWにするROWピン**: **ROW3**

*   **LEDの座標**: (6, 3)
    *   **LEDの番号**: 43
    *   **接続されている2つのROWピン**: ROW4, ROW6
    *   **HIGHにするROWピン**: **ROW4**
    *   **LOWにするROWピン**: **ROW6**

*   **LEDの座標**: (7, 3)
    *   **LEDの番号**: 44
    *   **接続されている2つのROWピン**: ROW6, ROW4
    *   **HIGHにするROWピン**: **ROW6**
    *   **LOWにするROWピン**: **ROW4**

*   **LEDの座標**: (8, 3)
    *   **LEDの番号**: 45
    *   **接続されている2つのROWピン**: ROW5, ROW6
    *   **HIGHにするROWピン**: **ROW5**
    *   **LOWにするROWピン**: **ROW6**

*   **LEDの座標**: (9, 3)
    *   **LEDの番号**: 46
    *   **接続されている2つのROWピン**: ROW6, ROW5
    *   **HIGHにするROWピン**: **ROW6**
    *   **LOWにするROWピン**: **ROW5**

*   **LEDの座標**: (10, 3)
    *   **LEDの番号**: 47
    *   **接続されている2つのROWピン**: ROW0, ROW7
    *   **HIGHにするROWピン**: **ROW0**
    *   **LOWにするROWピン**: **ROW7**

*   **LEDの座標**: (11, 3)
    *   **LEDの番号**: 48
    *   **接続されている2つのROWピン**: ROW7, ROW0
    *   **HIGHにするROWピン**: **ROW7**
    *   **LOWにするROWピン**: **ROW0**

*   **LEDの座標**: (0, 4)
    *   **LEDの番号**: 49
    *   **接続されている2つのROWピン**: ROW4, ROW7
    *   **HIGHにするROWピン**: **ROW4**
    *   **LOWにするROWピン**: **ROW7**

*   **LEDの座標**: (1, 4)
    *   **LEDの番号**: 50
    *   **接続されている2つのROWピン**: ROW7, ROW4
    *   **HIGHにするROWピン**: **ROW7**
    *   **LOWにするROWピン**: **ROW4**

*   **LEDの座標**: (2, 4)
    *   **LEDの番号**: 51
    *   **接続されている2つのROWピン**: ROW5, ROW7
    *   **HIGHにするROWピン**: **ROW5**
    *   **LOWにするROWピン**: **ROW7**

*   **LEDの座標**: (3, 4)
    *   **LEDの番号**: 52
    *   **接続されている2つのROWピン**: ROW7, ROW5
    *   **HIGHにするROWピン**: **ROW7**
    *   **LOWにするROWピン**: **ROW5**

*   **LEDの座標**: (4, 4)
    *   **LEDの番号**: 53
    *   **接続されている2つのROWピン**: ROW6, ROW7
    *   **HIGHにするROWピン**: **ROW6**
    *   **LOWにするROWピン**: **ROW7**

*   **LEDの座標**: (5, 4)
    *   **LEDの番号**: 54
    *   **接続されている2つのROWピン**: ROW7, ROW6
    *   **HIGHにするROWピン**: **ROW7**
    *   **LOWにするROWピン**: **ROW6**

*   **LEDの座標**: (6, 4)
    *   **LEDの番号**: 55
    *   **接続されている2つのROWピン**: ROW0, ROW8
    *   **HIGHにするROWピン**: **ROW0**
    *   **LOWにするROWピン**: **ROW8**

*   **LEDの座標**: (7, 4)
    *   **LEDの番号**: 56
    *   **接続されている2つのROWピン**: ROW8, ROW0
    *   **HIGHにするROWピン**: **ROW8**
    *   **LOWにするROWピン**: **ROW0**

*   **LEDの座標**: (8, 4)
    *   **LEDの番号**: 57
    *   **接続されている2つのROWピン**: ROW1, ROW8
    *   **HIGHにするROWピン**: **ROW1**
    *   **LOWにするROWピン**: **ROW8**

*   **LEDの座標**: (9, 4)
    *   **LEDの番号**: 58
    *   **接続されている2つのROWピン**: ROW8, ROW1
    *   **HIGHにするROWピン**: **ROW8**
    *   **LOWにするROWピン**: **ROW1**

*   **LEDの座標**: (10, 4)
    *   **LEDの番号**: 59
    *   **接続されている2つのROWピン**: ROW2, ROW8
    *   **HIGHにするROWピン**: **ROW2**
    *   **LOWにするROWピン**: **ROW8**

*   **LEDの座標**: (11, 4)
    *   **LEDの番号**: 60
    *   **接続されている2つのROWピン**: ROW8, ROW2
    *   **HIGHにするROWピン**: **ROW8**
    *   **LOWにするROWピン**: **ROW2**

*   **LEDの座標**: (0, 5)
    *   **LEDの番号**: 61
    *   **接続されている2つのROWピン**: ROW7, ROW4
    *   **HIGHにするROWピン**: **ROW7**
    *   **LOWにするROWピン**: **ROW4**

*   **LEDの座標**: (1, 5)
    *   **LEDの番号**: 62
    *   **接続されている2つのROWピン**: ROW4, ROW7
    *   **HIGHにするROWピン**: **ROW4**
    *   **LOWにするROWピン**: **ROW7**

*   **LEDの座標**: (2, 5)
    *   **LEDの番号**: 63
    *   **接続されている2つのROWピン**: ROW7, ROW5
    *   **HIGHにするROWピン**: **ROW7**
    *   **LOWにするROWピン**: **ROW5**

*   **LEDの座標**: (3, 5)
    *   **LEDの番号**: 64
    *   **接続されている2つのROWピン**: ROW5, ROW7
    *   **HIGHにするROWピン**: **ROW5**
    *   **LOWにするROWピン**: **ROW7**

*   **LEDの座標**: (4, 5)
    *   **LEDの番号**: 65
    *   **接続されている2つのROWピン**: ROW7, ROW6
    *   **HIGHにするROWピン**: **ROW7**
    *   **LOWにするROWピン**: **ROW6**

*   **LEDの座標**: (5, 5)
    *   **LEDの番号**: 66
    *   **接続されている2つのROWピン**: ROW6, ROW7
    *   **HIGHにするROWピン**: **ROW6**
    *   **LOWにするROWピン**: **ROW7**

*   **LEDの座標**: (6, 5)
    *   **LEDの番号**: 67
    *   **接続されている2つのROWピン**: ROW8, ROW0
    *   **HIGHにするROWピン**: **ROW8**
    *   **LOWにするROWピン**: **ROW0**

*   **LEDの座標**: (7, 5)
    *   **LEDの番号**: 68
    *   **接続されている2つのROWピン**: ROW0, ROW8
    *   **HIGHにするROWピン**: **ROW0**
    *   **LOWにするROWピン**: **ROW8**

*   **LEDの座標**: (8, 5)
    *   **LEDの番号**: 69
    *   **接続されている2つのROWピン**: ROW8, ROW1
    *   **HIGHにするROWピン**: **ROW8**
    *   **LOWにするROWピン**: **ROW1**

*   **LEDの座標**: (9, 5)
    *   **LEDの番号**: 70
    *   **接続されている2つのROWピン**: ROW1, ROW8
    *   **HIGHにするROWピン**: **ROW1**
    *   **LOWにするROWピン**: **ROW8**

*   **LEDの座標**: (10, 5)
    *   **LEDの番号**: 71
    *   **接続されている2つのROWピン**: ROW8, ROW2
    *   **HIGHにするROWピン**: **ROW8**
    *   **LOWにするROWピン**: **ROW2**

*   **LEDの座標**: (11, 5)
    *   **LEDの番号**: 72
    *   **接続されている2つのROWピン**: ROW2, ROW8
    *   **HIGHにするROWピン**: **ROW2**
    *   **LOWにするROWピン**: **ROW8**

*   **LEDの座標**: (0, 6)
    *   **LEDの番号**: 73
    *   **接続されている2つのROWピン**: ROW0, ROW9
    *   **HIGHにするROWピン**: **ROW0**
    *   **LOWにするROWピン**: **ROW9**

*   **LEDの座標**: (1, 6)
    *   **LEDの番号**: 74
    *   **接続されている2つのROWピン**: ROW9, ROW0
    *   **HIGHにするROWピン**: **ROW9**
    *   **LOWにするROWピン**: **ROW0**

*   **LEDの座標**: (2, 6)
    *   **LEDの番号**: 75
    *   **接続されている2つのROWピン**: ROW1, ROW9
    *   **HIGHにするROWピン**: **ROW1**
    *   **LOWにするROWピン**: **ROW9**

*   **LEDの座標**: (3, 6)
    *   **LEDの番号**: 76
    *   **接続されている2つのROWピン**: ROW9, ROW1
    *   **HIGHにするROWピン**: **ROW9**
    *   **LOWにするROWピン**: **ROW1**

*   **LEDの座標**: (4, 6)
    *   **LEDの番号**: 77
    *   **接続されている2つのROWピン**: ROW2, ROW9
    *   **HIGHにするROWピン**: **ROW2**
    *   **LOWにするROWピン**: **ROW9**

*   **LEDの座標**: (5, 6)
    *   **LEDの番号**: 78
    *   **接続されている2つのROWピン**: ROW9, ROW2
    *   **HIGHにするROWピン**: **ROW9**
    *   **LOWにするROWピン**: **ROW2**

*   **LEDの座標**: (6, 6)
    *   **LEDの番号**: 79
    *   **接続されている2つのROWピン**: ROW3, ROW9
    *   **HIGHにするROWピン**: **ROW3**
    *   **LOWにするROWピン**: **ROW9**

*   **LEDの座標**: (7, 6)
    *   **LEDの番号**: 80
    *   **接続されている2つのROWピン**: ROW9, ROW3
    *   **HIGHにするROWピン**: **ROW9**
    *   **LOWにするROWピン**: **ROW3**

*   **LEDの座標**: (8, 6)
    *   **LEDの番号**: 81
    *   **接続されている2つのROWピン**: ROW4, ROW9
    *   **HIGHにするROWピン**: **ROW4**
    *   **LOWにするROWピン**: **ROW9**

*   **LEDの座標**: (9, 6)
    *   **LEDの番号**: 82
    *   **接続されている2つのROWピン**: ROW9, ROW4
    *   **HIGHにするROWピン**: **ROW9**
    *   **LOWにするROWピン**: **ROW4**

*   **LEDの座標**: (10, 6)
    *   **LEDの番号**: 83
    *   **接続されている2つのROWピン**: ROW5, ROW9
    *   **HIGHにするROWピン**: **ROW5**
    *   **LOWにするROWピン**: **ROW9**

*   **LEDの座標**: (11, 6)
    *   **LEDの番号**: 84
    *   **接続されている2つのROWピン**: ROW9, ROW5
    *   **HIGHにするROWピン**: **ROW9**
    *   **LOWにするROWピン**: **ROW5**

*   **LEDの座標**: (0, 7)
    *   **LEDの番号**: 85
    *   **接続されている2つのROWピン**: ROW0, ROW10
    *   **HIGHにするROWピン**: **ROW0**
    *   **LOWにするROWピン**: **ROW10**

*   **LEDの座標**: (1, 7)
    *   **LEDの番号**: 86
    *   **接続されている2つのROWピン**: ROW10, ROW0
    *   **HIGHにするROWピン**: **ROW10**
    *   **LOWにするROWピン**: **ROW0**

*   **LEDの座標**: (2, 7)
    *   **LEDの番号**: 87
    *   **接続されている2つのROWピン**: ROW1, ROW10
    *   **HIGHにするROWピン**: **ROW1**
    *   **LOWにするROWピン**: **ROW10**

*   **LEDの座標**: (3, 7)
    *   **LEDの番号**: 88
    *   **接続されている2つのROWピン**: ROW10, ROW1
    *   **HIGHにするROWピン**: **ROW10**
    *   **LOWにするROWピン**: **ROW1**

*   **LEDの座標**: (4, 7)
    *   **LEDの番号**: 89
    *   **接続されている2つのROWピン**: ROW2, ROW10
    *   **HIGHにするROWピン**: **ROW2**
    *   **LOWにするROWピン**: **ROW10**

*   **LEDの座標**: (5, 7)
    *   **LEDの番号**: 90
    *   **接続されている2つのROWピン**: ROW10, ROW2
    *   **HIGHにするROWピン**: **ROW10**
    *   **LOWにするROWピン**: **ROW2**

*   **LEDの座標**: (6, 7)
    *   **LEDの番号**: 91
    *   **接続されている2つのROWピン**: ROW3, ROW10
    *   **HIGHにするROWピン**: **ROW3**
    *   **LOWにするROWピン**: **ROW10**

*   **LEDの座標**: (7, 7)
    *   **LEDの番号**: 92
    *   **接続されている2つのROWピン**: ROW10, ROW3
    *   **HIGHにするROWピン**: **ROW10**
    *   **LOWにするROWピン**: **ROW3**

*   **LEDの座標**: (8, 7)
    *   **LEDの番号**: 93
    *   **接続されている2つのROWピン**: ROW4, ROW10
    *   **HIGHにするROWピン**: **ROW4**
    *   **LOWにするROWピン**: **ROW10**

*   **LEDの座標**: (9, 7)
    *   **LEDの番号**: 94
    *   **接続されている2つのROWピン**: ROW10, ROW4
    *   **HIGHにするROWピン**: **ROW10**
    *   **LOWにするROWピン**: **ROW4**

*   **LEDの座標**: (10, 7)
    *   **LEDの番号**: 95
    *   **接続されている2つのROWピン**: ROW5, ROW10
    *   **HIGHにするROWピン**: **ROW5**
    *   **LOWにするROWピン**: **ROW10**

*   **LEDの座標**: (11, 7)
    *   **LEDの番号**: 96
    *   **接続されている2つのROWピン**: ROW10, ROW5
    *   **HIGHにするROWピン**: **ROW10**
    *   **LOWにするROWピン**: **ROW5**
"""

# データを空行で分割して各セクションに分ける
# filter(None, ...) は、分割によって生じた空の要素を削除する
sections = filter(None, data.strip().split('\n\n'))

rlist = []
rlist2 = []

#  d = {
#  # P0xx pins
#      "ROW0" : "P003_BIT",
#      "ROW1" : "P004_BIT",
#      "ROW2" : "P011_BIT",
#      "ROW3" : "P012_BIT",
#      "ROW4" : "P013_BIT",
#      "ROW5" : "P015_BIT",
#  # P2xx pins
#      "ROW6": "P204_BIT",
#      "ROW7": "P205_BIT",
#      "ROW8": "P206_BIT",
#      "ROW9": "P212_BIT",
#      "ROW10":"P213_BIT",
#  }


d = {
"ROW0": "P205_BIT",
"ROW1": "P012_BIT",
"ROW2": "P013_BIT",
"ROW3": "P003_BIT",
"ROW4": "P004_BIT",
"ROW5": "P011_BIT",
"ROW6": "P015_BIT",
"ROW7": "P204_BIT",
"ROW8": "P206_BIT",
"ROW9": "P212_BIT",
"ROW10": "P213_BIT",
}


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
    "P213_BIT",
    "P205_BIT",
    "P003_BIT",
    "P004_BIT",
    "P011_BIT",
    "P012_BIT",
    "P013_BIT",
    "P015_BIT",
    "P204_BIT",
    "P206_BIT",
    "P212_BIT",
]

#d = {i:j for i,j in zip(rows, rows_codes)}
pprint.pprint(d)

# 各セクションをループして情報を抽出
for i, section in enumerate(sections):
    # --- 正規表現を使って各行から数値を抽出 ---
    # `(\d+)` は1桁以上の数値をキャプチャするグループを意味します
    #print (section)

    coord_match = re.search(r"\*\s+\*\*LEDの座標\*\*: \((\d+), (\d+)\)", section)
    num_match = re.search(r"\*\s+\*\*LEDの番号\*\*: (\d+)", section)
    pins_match = re.search(r"\*\s+\*\*接続されている2つのROWピン\*\*: ROW(\d+), ROW(\d+)", section)
    high_match = re.search(r"\*\s+\*\*HIGHにするROWピン\*\*: \*\*ROW(\d+)\*\*", section)
    low_match = re.search(r"\*\s+\*\*LOWにするROWピン\*\*: \*\*ROW(\d+)\*\*", section)

    # すべての情報の抽出に成功した場合のみ処理を実行
    if all((coord_match, num_match, pins_match, high_match, low_match)):
        # 抽出した値を変数に格納
        coord_x, coord_y = coord_match.groups()
        led_num = num_match.group(1)
        pin1, pin2 = pins_match.groups()
        high_pin = high_match.group(1)
        low_pin = low_match.group(1)

        # 指定されたフォーマットで出力
        #print(f"--- セクション {i + 1} ---")
        #print(f"LEDの座標({coord_x}, {coord_y})")
        #print(f"LEDの番号: {led_num}")
        # ご提示のフォーマット `ROW%d, ROW0%d` は `ROW%d, ROW%d` の誤記と判断し修正しています
        #print(f"接続されている2つのROWピン: ROW{pin1}, ROW{pin2}")
        #print(f"HIGHにするROWピン: ROW{high_pin}")
        #print(f"LOWにするROWピン: ROW{low_pin}")
        rlist.append((led_num, low_pin, high_pin))
        rlist2.append((d[f"ROW{high_pin}"], d[f"ROW{low_pin}"]))
        #print("-" * 20) # セクションごとの区切り線


pprint.pprint(rlist)
pprint.pprint(rlist2)

print ("[")
for (i,j) in rlist2:
    print(f"    ({i}, {j}),")
print ("]")
