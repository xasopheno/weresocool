"""

Given two arrays of single digit integers, write a function that returns the sum of the two arrays. Do so without converting the arrays into either integers or strings.
Example: [1, 2, 3, 4] and [5, 6, 7] -> [1, 8, 0, 1]


"""


def adder(num1, num2):
    result = []
    num1.reverse()
    num2.reverse()
    print(num1, num2)

    idx = 0
    while True:
        if idx < max(len(num1), len(num2)):
            break
        val = 0
        carry = 0
        for num in [num1, num2]:
            if num[idx] > len(num):
                continue
            val += num

        val += carry
        if val > 10:
            carry = carry % 10
            val = 0

        result += [val]
        idx += 1

    result.reverse()
    return result


num1 = [1, 2, 3, 4]
num2 = [5, 6, 7]
expected = [1, 8, 0, 1]
result = adder(num1, num2)
print(result, expected)
print(result == expected)
