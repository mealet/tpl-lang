import "bool.tpl"; // should compile `bool.tpl`  module and execute

import "foo.tpl"; // should import function from module

int result = foo(7, 3);

print(result); // should print 10