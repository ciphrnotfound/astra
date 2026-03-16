```python
import unittest

class TestFunctions(unittest.TestCase):

    def test_greet_valid_name(self):
        self.assertEqual(greet("John"), "Hello, John")

    def test_greet_empty_name(self):
        with self.assertRaises(TypeError):
            greet("")

    def test_greet_non_string_name(self):
        with self.assertRaises(TypeError):
            greet(123)

    def test_add_valid_numbers(self):
        self.assertEqual(add(1, 2), 3)

    def test_add_negative_numbers(self):
        self.assertEqual(add(-1, -2), -3)

    def test_add_large_numbers(self):
        self.assertEqual(add(1000000, 2000000), 3000000)

    def test_add_non_integer_values(self):
        with self.assertRaises(TypeError):
            add("a", 1)
        with self.assertRaises(TypeError):
            add(1, "a")

    def test_format_user_valid_id(self):
        self.assertEqual(format_user(1, "John"), "1:john")

    def test_format_user_empty_username(self):
        with self.assertRaises(TypeError):
            format_user(1, "")

    def test_format_user_non_string_username(self):
        with self.assertRaises(TypeError):
            format_user(1, 123)

    def test_format_user_negative_id(self):
        self.assertEqual(format_user(-1, "John"), "-1:john")

    def test_format_user_zero_id(self):
        self.assertEqual(format_user(0, "John"), "0:john")

if __name__ == "__main__":
    unittest.main()
```