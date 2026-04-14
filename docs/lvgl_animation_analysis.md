# LVGL 动画系统深度分析

## 1. 概述

LVGL (Light and Versatile Graphics Library) 的动画系统是一个专为嵌入式系统设计的轻量级动画引擎。它采用 C 语言编写，具有内存占用小、执行效率高的特点。

### 核心特性

- **纯软件渲染**：不依赖 GPU，适用于所有嵌入式平台
- **定时器驱动**：基于 `lv_timer` 的动画更新机制
- **路径插值**：多种内置缓动曲线（线性、缓入、缓出、弹跳等）
- **链式动画**：支持动画完成回调，便于创建动画序列
- **回播功能**：支持正向播放后自动反向播放
- **重复播放**：支持无限循环或指定次数循环
- **延迟启动**：支持动画延迟执行

---

## 2. 核心组件架构

### 2.1 动画结构体 (lv_anim_t)

```c
/** Describes an animation */
typedef struct _lv_anim_t {
    void * var;                          /**< Variable to animate */
    lv_anim_exec_xcb_t exec_cb;          /**< Function to execute to animate */
    lv_anim_start_cb_t start_cb;         /**< Call when animation starts */
    lv_anim_ready_cb_t ready_cb;         /**< Call when animation is ready */
    lv_anim_deleted_cb_t deleted_cb;     /**< Call when animation is deleted */
    lv_anim_get_value_cb_t get_value_cb; /**< Get current value in relative mode */
    void * user_data;                    /**< Custom user data */
    
    lv_anim_path_cb_t path_cb;           /**< Describe the path (curve) */
    int32_t start_value;                 /**< Start value */
    int32_t current_value;               /**< Current value */
    int32_t end_value;                   /**< End value */
    int32_t time;                        /**< Animation time in ms */
    int32_t act_time;                    /**< Current time (negative = delay) */
    
    uint32_t playback_delay;             /**< Wait before play back */
    uint32_t playback_time;              /**< Duration of playback */
    uint32_t repeat_delay;               /**< Wait before repeat */
    uint16_t repeat_cnt;                 /**< Repeat count */
    
    uint8_t early_apply  : 1;            /**< Apply start value immediately */
    uint8_t playback_now : 1;            /**< Play back is in progress */
    uint8_t run_round : 1;               /**< Animation has run in this round */
    uint8_t start_cb_called : 1;         /**< start_cb was already called */
} lv_anim_t;
```

**关键字段说明：**

| 字段 | 说明 |
|------|------|
| `var` | 要动画化的变量指针 |
| `exec_cb` | 动画执行回调，如 `lv_obj_set_x` |
| `path_cb` | 路径/曲线回调，如 `lv_anim_path_ease_out` |
| `act_time` | 当前时间，负值表示延迟 |
| `early_apply` | 是否立即应用起始值 |
| `playback_now` | 是否正在回播 |

### 2.2 回调函数类型

```c
/** Get the current value during an animation */
typedef int32_t (*lv_anim_path_cb_t)(const struct _lv_anim_t *);

/** Animator function prototype */
typedef void (*lv_anim_exec_xcb_t)(void *, int32_t);

/** Custom exec callback */
typedef void (*lv_anim_custom_exec_cb_t)(struct _lv_anim_t *, int32_t);

/** Callback when animation is ready */
typedef void (*lv_anim_ready_cb_t)(struct _lv_anim_t *);

/** Callback when animation starts */
typedef void (*lv_anim_start_cb_t)(struct _lv_anim_t *);

/** Callback when animation is deleted */
typedef void (*lv_anim_deleted_cb_t)(struct _lv_anim_t *);

/** Get current value callback (for relative animations) */
typedef int32_t (*lv_anim_get_value_cb_t)(struct _lv_anim_t *);
```

---

## 3. 动画路径/曲线系统

### 3.1 内置路径函数

```c
/** Linear animation */
int32_t lv_anim_path_linear(const lv_anim_t * a);

/** Ease-in animation */
int32_t lv_anim_path_ease_in(const lv_anim_t * a);

/** Ease-out animation */
int32_t lv_anim_path_ease_out(const lv_anim_t * a);

/** Ease-in-out animation */
int32_t lv_anim_path_ease_in_out(const lv_anim_t * a);

/** Overshoot animation */
int32_t lv_anim_path_overshoot(const lv_anim_t * a);

/** Bounce animation */
int32_t lv_anim_path_bounce(const lv_anim_t * a);

/** Step animation (no interpolation) */
int32_t lv_anim_path_step(const lv_anim_t * a);
```

### 3.2 路径计算原理

以线性路径为例：

```c
int32_t lv_anim_path_linear(const lv_anim_t * a)
{
    /* Calculate current step: map act_time to [0, 1024] */
    int32_t step = lv_map(a->act_time, 0, a->time, 0, LV_ANIM_RESOLUTION);
    
    /* Calculate new value proportionally */
    int32_t new_value;
    new_value = step * (a->end_value - a->start_value);
    new_value = new_value >> LV_ANIM_RES_SHIFT;  /* Divide by 1024 */
    new_value += a->start_value;
    
    return new_value;
}
```

**关键常量：**
- `LV_ANIM_RESOLUTION = 1024` - 动画精度分辨率
- `LV_ANIM_RES_SHIFT = 10` - 右移位数（相当于除以1024）

### 3.3 贝塞尔曲线实现

```c
int32_t lv_anim_path_ease_out(const lv_anim_t * a)
{
    /* Map time to [0, LV_BEZIER_VAL_MAX] */
    uint32_t t = lv_map(a->act_time, 0, a->time, 0, LV_BEZIER_VAL_MAX);
    
    /* Cubic bezier: P0=0, P1=900, P2=950, P3=1024 */
    int32_t step = lv_bezier3(t, 0, 900, 950, LV_BEZIER_VAL_MAX);
    
    /* Apply to value range */
    int32_t new_value;
    new_value = step * (a->end_value - a->start_value);
    new_value = new_value >> LV_BEZIER_VAL_SHIFT;
    new_value += a->start_value;
    
    return new_value;
}
```

**贝塞尔控制点：**
- `ease_in`: (0, 50, 100, 1024) - 开始缓慢
- `ease_out`: (0, 900, 950, 1024) - 结束缓慢
- `ease_in_out`: (0, 50, 952, 1024) - 两端缓慢
- `overshoot`: (0, 1000, 1300, 1024) - 超出回弹

---

## 4. 动画执行机制

### 4.1 定时器驱动架构

```
┌─────────────────────────────────────────────────────────────┐
│                     LVGL Timer System                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  lv_timer_handler()  ←── 由主循环定期调用                     │
│       │                                                      │
│       ▼                                                      │
│  ┌─────────────────┐                                         │
│  │ 遍历所有 timer  │                                         │
│  └────────┬────────┘                                         │
│           │                                                  │
│           ▼                                                  │
│  ┌─────────────────┐                                         │
│  │ anim_timer()    │  ←── 动画系统定时器回调                  │
│  │ (period: 16ms)  │                                         │
│  └────────┬────────┘                                         │
│           │                                                  │
│           ▼                                                  │
│  ┌─────────────────┐                                         │
│  │ 遍历所有动画     │                                         │
│  │ 更新 act_time   │                                         │
│  │ 计算新值        │                                         │
│  │ 执行 exec_cb    │  ←── 如 lv_obj_set_x(obj, new_x)        │
│  └─────────────────┘                                         │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 4.2 动画定时器实现

```c
static void anim_timer(lv_timer_t * param)
{
    uint32_t elaps = lv_tick_elaps(last_timer_run);
    
    /* Flip run round flag */
    anim_run_round = anim_run_round ? false : true;
    
    lv_anim_t * a = _lv_ll_get_head(&LV_GC_ROOT(_lv_anim_ll));
    
    while(a != NULL) {
        anim_list_changed = false;
        
        if(a->run_round != anim_run_round) {
            a->run_round = anim_run_round;
            
            /* Handle delay */
            int32_t new_act_time = a->act_time + elaps;
            if(!a->start_cb_called && a->act_time <= 0 && new_act_time >= 0) {
                if(a->start_cb) a->start_cb(a);
                a->start_cb_called = 1;
            }
            
            a->act_time += elaps;
            
            if(a->act_time >= 0) {
                if(a->act_time > a->time) a->act_time = a->time;
                
                /* Calculate new value using path */
                int32_t new_value = a->path_cb(a);
                
                if(new_value != a->current_value) {
                    a->current_value = new_value;
                    /* Apply the value */
                    if(a->exec_cb) a->exec_cb(a->var, new_value);
                }
                
                /* Animation finished? */
                if(a->act_time >= a->time) {
                    anim_ready_handler(a);
                }
            }
        }
        
        /* Get next, but check if list changed */
        if(anim_list_changed) {
            a = _lv_ll_get_head(&LV_GC_ROOT(_lv_anim_ll));
        } else {
            a = _lv_ll_get_next(&LV_GC_ROOT(_lv_anim_ll), a);
        }
    }
    
    last_timer_run = lv_tick_get();
}
```

### 4.3 动画完成处理

```c
static void anim_ready_handler(lv_anim_t * a)
{
    /* Handle playback (reverse direction) */
    if(a->playback_time != 0 && !a->playback_now) {
        /* Swap start and end values */
        int32_t tmp = a->start_value;
        a->start_value = a->end_value;
        a->end_value = tmp;
        
        a->playback_now = 1;
        a->act_time = -(int32_t)a->playback_delay;
        a->time = a->playback_time;
    }
    /* Handle repeat */
    else if(a->repeat_cnt > 1 || a->repeat_cnt == LV_ANIM_REPEAT_INFINITE) {
        if(a->repeat_cnt != LV_ANIM_REPEAT_INFINITE) {
            a->repeat_cnt--;
        }
        
        /* Restore original values after playback */
        if(a->playback_time != 0) {
            int32_t tmp = a->start_value;
            a->start_value = a->end_value;
            a->end_value = tmp;
        }
        
        a->playback_now = 0;
        a->act_time = -(int32_t)a->repeat_delay;
    }
    /* Animation truly finished */
    else {
        if(a->ready_cb) a->ready_cb(a);
        lv_anim_del(a->var, a->exec_cb);
    }
}
```

---

## 5. 使用示例

### 5.1 基础动画

```c
/* Animate object position */
lv_anim_t a;
lv_anim_init(&a);
lv_anim_set_var(&a, obj);
lv_anim_set_exec_cb(&a, (lv_anim_exec_xcb_t)lv_obj_set_x);
lv_anim_set_values(&a, 0, 100);           /* Start: 0, End: 100 */
lv_anim_set_time(&a, 500);                /* Duration: 500ms */
lv_anim_set_path_cb(&a, lv_anim_path_ease_out);
lv_anim_start(&a);
```

### 5.2 带延迟的动画

```c
lv_anim_t a;
lv_anim_init(&a);
lv_anim_set_var(&a, obj);
lv_anim_set_exec_cb(&a, (lv_anim_exec_xcb_t)lv_obj_set_y);
lv_anim_set_values(&a, 0, 200);
lv_anim_set_time(&a, 1000);
lv_anim_set_delay(&a, 300);               /* Delay 300ms before starting */
lv_anim_start(&a);
```

### 5.3 循环动画

```c
lv_anim_t a;
lv_anim_init(&a);
lv_anim_set_var(&a, obj);
lv_anim_set_exec_cb(&a, (lv_anim_exec_xcb_t)lv_obj_set_x);
lv_anim_set_values(&a, 0, 100);
lv_anim_set_time(&a, 500);
lv_anim_set_repeat_count(&a, LV_ANIM_REPEAT_INFINITE);
lv_anim_set_repeat_delay(&a, 100);        /* 100ms delay between repeats */
lv_anim_start(&a);
```

### 5.4 回播动画

```c
lv_anim_t a;
lv_anim_init(&a);
lv_anim_set_var(&a, obj);
lv_anim_set_exec_cb(&a, (lv_anim_exec_xcb_t)lv_obj_set_x);
lv_anim_set_values(&a, 0, 100);
lv_anim_set_time(&a, 500);
lv_anim_set_playback_time(&a, 500);       /* Play back in 500ms */
lv_anim_set_playback_delay(&a, 200);      /* 200ms delay before playback */
lv_anim_start(&a);
/* Result: 0→100 (500ms) → wait 200ms → 100→0 (500ms) */
```

### 5.5 动画序列（链式回调）

```c
static void anim_x_cb(lv_anim_t * a)
{
    /* Start Y animation when X animation completes */
    lv_obj_t * obj = a->var;
    
    lv_anim_t b;
    lv_anim_init(&b);
    lv_anim_set_var(&b, obj);
    lv_anim_set_exec_cb(&b, (lv_anim_exec_xcb_t)lv_obj_set_y);
    lv_anim_set_values(&b, 0, 100);
    lv_anim_set_time(&b, 500);
    lv_anim_start(&b);
}

/* Start first animation */
lv_anim_t a;
lv_anim_init(&a);
lv_anim_set_var(&a, obj);
lv_anim_set_exec_cb(&a, (lv_anim_exec_xcb_t)lv_obj_set_x);
lv_anim_set_values(&a, 0, 100);
lv_anim_set_time(&a, 500);
lv_anim_set_ready_cb(&a, anim_x_cb);      /* Chain to next animation */
lv_anim_start(&a);
```

### 5.6 相对动画

```c
/* Animate from current position */
lv_anim_t a;
lv_anim_init(&a);
lv_anim_set_var(&a, obj);
lv_anim_set_exec_cb(&a, (lv_anim_exec_xcb_t)lv_obj_set_x);
lv_anim_set_values(&a, 0, 50);            /* Relative: move 50px right */
lv_anim_set_time(&a, 500);
lv_anim_set_get_value_cb(&a, lv_obj_get_x); /* Get current X */
lv_anim_start(&a);
```

---

## 6. 内存管理

### 6.1 动画存储

```c
/* Animation linked list (global) */
_lv_ll_init(&LV_GC_ROOT(_lv_anim_ll), sizeof(lv_anim_t));

/* Animation timer */
_lv_anim_tmr = lv_timer_create(anim_timer, LV_DISP_DEF_REFR_PERIOD, NULL);
```

### 6.2 删除动画

```c
/* Delete specific animation */
lv_anim_del(obj, (lv_anim_exec_xcb_t)lv_obj_set_x);

/* Delete all animations of an object */
lv_anim_del(obj, NULL);

/* Delete all animations */
lv_anim_del_all(void);
```

### 6.3 获取动画

```c
/* Get animation count */
uint16_t count = lv_anim_count_running();

/* Get specific animation */
lv_anim_t * anim = lv_anim_get(obj, (lv_anim_exec_xcb_t)lv_obj_set_x);

/* Get remaining playtime */
uint32_t playtime = lv_anim_get_playtime(anim);
```

---

## 7. 架构设计亮点

### 7.1 纯回调驱动

- 不直接操作对象属性
- 通过 `exec_cb` 回调函数应用值
- 支持任意属性的动画化

### 7.2 时间驱动而非帧驱动

- 使用 `lv_tick_get()` 获取实际时间
- 动画速度不受帧率影响
- 自动补偿帧率波动

### 7.3 双向链表存储

- 使用 LVGL 的 `_lv_ll` 链表
- 支持动态添加/删除
- 遍历安全（支持遍历时修改）

### 7.4 低内存占用

- 动画结构体紧凑（约 64 字节）
- 无动态内存分配（除了链表节点）
- 支持静态分配

---

## 8. 与 Bevy 动画系统对比

| 特性 | LVGL Animation | Bevy Animation |
|------|----------------|----------------|
| 语言 | C | Rust |
| 架构 | 回调驱动 | ECS |
| 存储 | 双向链表 | Component |
| 曲线 | 函数指针 | Trait |
| 混合 | 不支持 | 支持 (AnimationGraph) |
| 过渡 | 手动链式 | AnimationTransitions |
| 目标平台 | 嵌入式 | 桌面/移动/主机 |
| 内存占用 | ~64 bytes/anim | 较大 |
| GPU 加速 | 不支持 | 可选 |

---

## 9. 总结

LVGL 的动画系统是一个为嵌入式系统优化的轻量级实现：

1. **简单高效**：纯 C 实现，无复杂依赖
2. **灵活回调**：通过回调函数支持任意属性动画
3. **时间准确**：基于实际时间计算，不受帧率影响
4. **功能完整**：支持延迟、回播、循环、链式动画
5. **内存友好**：紧凑的数据结构，低内存占用

该系统非常适合资源受限的嵌入式设备，如 MCU、IoT 设备等。虽然功能不如 Bevy 等现代游戏引擎丰富，但在其目标场景下提供了恰到好处的动画能力。
