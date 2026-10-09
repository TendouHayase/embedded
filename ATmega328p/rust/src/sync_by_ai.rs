use core::{
    arch::asm,
    cell::{Cell, UnsafeCell},
    mem::MaybeUninit,
    pin::Pin,
    ptr::{self, NonNull, read_volatile},
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

mod register {
    pub const SREG: *mut u8 = 0x5F as *mut u8;
}

mod mask {
    pub const GLOBAL_INTERRUPT_MASK: u8 = 1 << 7;
}

const DATA_CONTAINER_SIZE: usize = 32;
const MAX_TASK: usize = 8;

#[inline(always)]
fn interrupt_free<R>(f: impl FnOnce() -> R) -> R {
    let is_enabled = unsafe { read_volatile(register::SREG) } & mask::GLOBAL_INTERRUPT_MASK != 0;
    unsafe { asm!("cli") };
    let r = f();
    if is_enabled {
        unsafe {
            asm!("sei");
        }
    }
    r
}

struct CsCell<T>(UnsafeCell<T>);

unsafe impl<T> Sync for CsCell<T> {}

impl<T> CsCell<T> {
    const fn new(v: T) -> Self {
        Self(UnsafeCell::new(v))
    }
    fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        interrupt_free(|| f(unsafe { &mut *self.0.get() }))
    }
}

#[repr(transparent)]
struct SyncWrapper<T>(T);

impl<T> SyncWrapper<T> {
    pub const fn new(value: T) -> Self {
        Self(value)
    }
}

unsafe impl<T> Sync for SyncWrapper<T> {}

pub fn run() -> ! {
    loop {
        match QUEUE.with(|q| q.pop()) {
            Some(id) => poll_task(id),
            None => idle(),
        }
    }
}

pub fn poll_task(id: TaskId) {
    let task = &TASK_ARENA.0[id as usize];
    let Some(fut) = task.future.get() else { return };

    let waker = waker_for(id);
    let mut ctx = Context::from_waker(&waker);

    let pinned = unsafe { Pin::new_unchecked(&mut *fut.as_ptr()) };

    if pinned.poll(&mut ctx).is_ready() {
        unsafe { ptr::drop_in_place(fut.as_ptr()) };
        task.future.set(None);
    }
}

fn idle() {
    unsafe { asm!("cli") }
    let is_empty = QUEUE.with(|q| q.is_empty());
    if is_empty {
        unsafe { asm!("sei", "sleep") };
    } else {
        unsafe { asm!("sei") };
    }
}

struct Task {
    data: UnsafeCell<[MaybeUninit<u8>; DATA_CONTAINER_SIZE]>,
    future: Cell<Option<NonNull<dyn Future<Output = ()>>>>,
}

impl Task {
    const EMPTY: Self = Self {
        data: UnsafeCell::new([MaybeUninit::uninit(); DATA_CONTAINER_SIZE]),
        future: Cell::new(None),
    };
}

static TASK_ARENA: SyncWrapper<[Task; MAX_TASK]> = SyncWrapper::new([Task::EMPTY; MAX_TASK]);

type TaskId = u8;

pub fn spawn<F: Future<Output = ()> + 'static>(fut: F) {
    const {
        assert!(size_of::<F>() <= DATA_CONTAINER_SIZE, "task too large");
    }

    let (id, task) = TASK_ARENA
        .0
        .iter()
        .enumerate()
        .find(|(_, t)| t.future.get().is_none())
        .expect("no free task slot");

    let data_ptr = task.data.get() as *mut F;
    unsafe {
        data_ptr.write(fut);
    }
    let fat_ptr: *mut dyn Future<Output = ()> = data_ptr;
    task.future.set(NonNull::new(fat_ptr));

    QUEUE.with(|q| q.push(id as TaskId));
}

struct RunningQueue {
    buf: [TaskId; Self::QUEUE_SIZE],
    head: usize,
    len: usize,
    queued: [bool; MAX_TASK],
}

impl RunningQueue {
    const QUEUE_SIZE: usize = 32;
    const QUEUE_MASK: usize = Self::QUEUE_SIZE - 1;

    pub const fn new() -> Self {
        Self {
            buf: [0; Self::QUEUE_SIZE],
            head: 0,
            len: 0,
            queued: [false; MAX_TASK],
        }
    }

    pub fn push(&mut self, id: TaskId) {
        let i = id as usize;
        if self.queued[i] {
            return;
        }

        self.queued[i] = true;
        self.buf[(self.head + self.len) & Self::QUEUE_MASK] = id;
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<TaskId> {
        if self.is_empty() {
            return None;
        }
        let id = self.buf[self.head];
        self.head = (self.head + 1) & Self::QUEUE_MASK;
        self.len -= 1;
        self.queued[id as usize] = false;

        Some(id)
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

static QUEUE: CsCell<RunningQueue> = CsCell::new(RunningQueue::new());

static WAKER_VTABLE: RawWakerVTable =
    RawWakerVTable::new(clone_waker, wake, wake_by_ref, drop_waker);

fn raw_waker(id: TaskId) -> RawWaker {
    RawWaker::new(ptr::without_provenance(id as usize), &WAKER_VTABLE)
}

fn waker_for(id: TaskId) -> Waker {
    unsafe { Waker::from_raw(raw_waker(id)) }
}

unsafe fn clone_waker(data: *const ()) -> RawWaker {
    RawWaker::new(data, &WAKER_VTABLE)
}

unsafe fn wake(data: *const ()) {
    unsafe { wake_by_ref(data) }
}

unsafe fn wake_by_ref(data: *const ()) {
    let id = data.addr() as TaskId;
    QUEUE.with(|q| q.push(id));
}

unsafe fn drop_waker(_data: *const ()) {}

struct Timer {
    used: bool,
    deadline: u16,
    waker: Option<Waker>,
}

impl Timer {
    const EMPTY: Self = Self {
        used: false,
        deadline: 0,
        waker: None,
    };
}

struct Timers {
    now: u16, //ms
    entries: [Timer; Self::MAX_TIMERS],
}

impl Timers {
    pub const MAX_TIMERS: usize = 4;
}

static TIMERS: CsCell<Timers> = CsCell::new(Timers {
    now: 0,
    entries: [Timer::EMPTY; Timers::MAX_TIMERS],
});

fn reached(now: u16, deadline: u16) -> bool {
    now.wrapping_sub(deadline) < 0x8000
}

pub fn now_ms() -> u16 {
    TIMERS.with(|t| t.now)
}

#[unsafe(no_mangle)]
pub extern "avr-interrupt" fn __vector_11() {
    TIMERS.with(|t| {
        t.now = t.now.wrapping_add(1);
        let now = t.now;
        for e in t.entries.iter_mut() {
            if e.used
                && reached(now, e.deadline)
                && let Some(w) = e.waker.take()
            {
                w.wake();
            }
        }
    });
}

pub struct Delay {
    ms: u16,
    deadline: u16,
    entry: Option<usize>,
}

impl Delay {
    pub fn ms(ms: u16) -> Self {
        Self {
            ms,
            deadline: 0,
            entry: None,
        }
    }
}

impl Future for Delay {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        TIMERS.with(|t| match this.entry {
            None => {
                this.deadline = t.now.wrapping_add(this.ms);
                if reached(t.now, this.deadline) {
                    return Poll::Ready(());
                }

                let i = t
                    .entries
                    .iter()
                    .position(|e| !e.used)
                    .expect("no free timer");

                t.entries[i] = Timer {
                    used: true,
                    deadline: this.deadline,
                    waker: Some(cx.waker().clone()),
                };

                this.entry = Some(i);
                Poll::Pending
            }

            Some(i) => {
                if reached(t.now, this.deadline) {
                    t.entries[i] = Timer::EMPTY;
                    this.entry = None;
                    Poll::Ready(())
                } else {
                    let e = &mut t.entries[i];
                    let same = matches!(&e.waker, Some(w) if w.will_wake(cx.waker()));
                    if !same {
                        e.waker = Some(cx.waker().clone());
                    }

                    Poll::Pending
                }
            }
        })
    }
}

impl Drop for Delay {
    fn drop(&mut self) {
        if let Some(i) = self.entry {
            TIMERS.with(|t| t.entries[i] = Timer::EMPTY);
        }
    }
}
