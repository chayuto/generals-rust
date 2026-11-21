#pragma once
#include <cstdint>

template <typename CounterT>
class RefCountValue {
    mutable CounterT m_refs = 1;
public:
    void Add_Ref() const { ++m_refs; }
    template <typename DelFn, typename Self>
    void Release_Ref(DelFn del, Self* self) const {
        if (--m_refs == 0) del(self);
    }
    CounterT Num_Refs() const { return m_refs; }
};

class MemoryPoolObject {
public:
    virtual ~MemoryPoolObject() = default;
    static void deleteInstanceInternal(const MemoryPoolObject* p) { delete p; }
};

template <typename T>
class RefCountPtr {
    T* m_ptr = nullptr;
public:
    RefCountPtr() = default;
    explicit RefCountPtr(T* p) : m_ptr(p) {
        if (m_ptr) m_ptr->Add_Ref();
    }
    RefCountPtr(const RefCountPtr& o) : m_ptr(o.m_ptr) {
        if (m_ptr) m_ptr->Add_Ref();
    }
    ~RefCountPtr() {
        if (m_ptr) m_ptr->Release_Ref();
    }
};

#define MEMORY_POOL_GLUE_ABC(cls) \
public: \
    virtual ~cls();

#define MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(cls, poolName) \
public: \
    virtual ~cls();

#define EMPTY_DTOR(cls) inline cls::~cls() {}

#ifndef newInstance
#define newInstance(cls) (new cls)
#endif

inline void deleteInstance(MemoryPoolObject* p) {
    delete p;
}
