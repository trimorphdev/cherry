# alloc
`alloc`, `realloc` and `dealloc` are keywords which intend to make manual memory management safer in Cherry.

They would use the global allocator, in a system inspired by Rust:

```cherry
#![global_allocator(MyGlobalAllocator)]

export struct MyGlobalAllocator {
    // ...
}

impl GlobalAllocator for MyGlobalAllocator {
    // ...
}
```

## Using `alloc`
The `alloc` keyword should allow multiple syntaxes, allowing easy allocation and management of memory:

```
let my_ptr = alloc <usize>; // my_ptr: Ptr<usize>
// or
let my_ptr = alloc 8 as Ptr<usize>; // my_ptr: Ptr<usize>
```

Or you can allocate a pointer for multiple items at once:

```
let my_ptr = alloc [usize; 2]; // my_ptr.size() == 16
```

`RawPtr<T>`s are raw, memory unmanaged pointers that must manually be managed by the user.

Cherry pointers (`Ptr<T>`) are automatically memory managed by default, though there are some requirements for this to work.

Data structures which need to be memory managed, like `Vec`s, `HashMap`s, etc. must have the correct type when allocated.  For example, `my_ptr` in this example is properly memory managed because it has the correct type.

```
let my_ptr = alloc <Vec<usize>>; // my_ptr: Ptr<Vec<usize>>
my_ptr[0] = Vec.new();
```

But in this example, `my_ptr` does NOT have the correct type, and only the top-level pointer is deallocated.  The pointer that the `Vec` itself is never deconstructed and this causes a memory leak:

```
let my_ptr = alloc size_of<Vec<usize>>(); // my_ptr: Ptr<uint8>
my_ptr[0] = Vec.new();
```

Now there is a way to combat this, we can verify that `my_ptr` is the correct type at compile time to help with this problem.

```
let my_ptr = alloc size_of<Vec<usize>>(); // my_ptr: Ptr<uint8>
my_ptr[0] = Vec.new(); // error: cannot set value of Ptr<uint8> to Vec<T>.
```

## `dealloc` and `realloc`

`dealloc` manually deallocates memory, and with Cherry's automated memory management, it is usually only useful with `RawPtr`s, rather than `Ptr`s.

```
let my_ptr = alloc 1;
dealloc my_ptr;
```

`realloc` on the other hand is useful for both kinds of pointers, as it simply reallocates the pointer.

```
let my_ptr = alloc 1;
my_ptr = realloc my_ptr => 2;
// my_ptr.size() == 2
```

And, `realloc` allocates the right amount of memory for the provided type; for example:

```
let my_ptr = alloc <uint64>; // my_ptr.size() == 8
my_ptr = realloc my_ptr => 2;
// my_ptr.size() == 16
```

## `Ptr`s and `RawPtr`s
`Ptr`s and `RawPtr`s keep track of size and align, allowing for more safe use of direct memory access.