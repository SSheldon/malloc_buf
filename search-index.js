var searchIndex = {};
searchIndex['malloc_buf'] = {"items":[[0,"","malloc_buf",""],[1,"MallocBuffer","","A type that represents a `malloc`'d chunk of memory."],[1,"MallocString","","A type that represents a `malloc`'d string."],[10,"new","","Constructs a new `MallocBuffer` for a `malloc`'d buffer\nwith the given length at the given pointer.\nReturns `None` if the given pointer is null.",0],[10,"drop","","",0],[10,"as_slice","","",0],[10,"new","","Constructs a new `MallocString` for a `malloc`'d C string buffer.\nReturns `None` if the given pointer is null or the C string isn't UTF8.\nWhen this `MallocString` drops, the buffer will be `free`'d.",1],[10,"as_slice","","",1]],"paths":[[1,"MallocBuffer"],[1,"MallocString"]]};

initSearch(searchIndex);
