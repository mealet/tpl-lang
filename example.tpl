fn<int64> fib = int64 ( int64 index ) {
  int64 left = 0;
  int64 right = 1;
  int64 result = 0;

  for i in index {
    result = left + right;
    left = right;
    right = result;
  };

  return result;
};

int64 result = fib(1000);
print(result);
