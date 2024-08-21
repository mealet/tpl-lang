int left = 0;
int right = 1;
int result = 1;

for i in 50 {
  result = left + right;
  left = right;
  right = result;

  print(result);
};
