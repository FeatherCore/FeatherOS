# LVGL 输入系统分析文档

## 1. 概述

LVGL（Light and Versatile Graphics Library）输入系统负责处理用户交互事件，包括触摸、鼠标、键盘、编码器和外部按钮等输入设备。

### 1.1 核心组件

| 组件 | 文件路径 | 功能 |
|------|---------|------|
| 输入设备核心 | `src/indev/lv_indev.c` | 输入设备管理、事件分发 |
| 输入设备私有头 | `src/indev/lv_indev_private.h` | 内部数据结构定义 |
| 输入设备公共API | `src/indev/lv_indev.h` | 对外接口 |
| 滚动处理 | `src/indev/lv_indev_scroll.c` | 滚动和惯性滚动处理 |
| 手势识别 | `src/indev/lv_indev_gesture.c` | 多点触控手势识别 |
| evdev驱动 | `src/drivers/evdev/lv_evdev.c` | Linux evdev接口驱动 |

---

## 2. 输入设备类型

LVGL 支持以下输入设备类型（`lv_indev_type_t`）：

```c
typedef enum {
    LV_INDEV_TYPE_NONE,    // 未初始化状态
    LV_INDEV_TYPE_POINTER, // 触摸板、鼠标、外部按钮
    LV_INDEV_TYPE_KEYPAD,  // 键盘
    LV_INDEV_TYPE_BUTTON,  // 外部硬件按钮（映射到屏幕特定点）
    LV_INDEV_TYPE_ENCODER, // 编码器（左右旋转+按钮）
} lv_indev_type_t;
```

### 2.1 输入状态

```c
typedef enum {
    LV_INDEV_STATE_RELEASED = 0,
    LV_INDEV_STATE_PRESSED
} lv_indev_state_t;
```

### 2.2 运行模式

```c
typedef enum {
    LV_INDEV_MODE_NONE = 0,
    LV_INDEV_MODE_TIMER,  // 定时器模式（默认）
    LV_INDEV_MODE_EVENT,   // 事件驱动模式
} lv_indev_mode_t;
```

---

## 3. 输入设备数据结构

### 3.1 `lv_indev_t` 核心结构体

```c
struct _lv_indev_t {
    lv_indev_type_t type;              // 输入设备类型
    lv_indev_read_cb_t read_cb;       // 读取回调函数指针
    lv_indev_state_t state;           // 当前状态
    lv_indev_state_t prev_state;      // 之前的状态
    lv_indev_mode_t mode;             // 运行模式

    // 时间戳
    uint32_t timestamp;                // 最后事件时间戳
    uint32_t pr_timestamp;             // 按下时间戳
    uint32_t longpr_rep_timestamp;     // 长按重复时间戳

    // 显示关联
    lv_display_t * disp;              // 关联的显示

    // 定时器读取
    lv_timer_t * read_timer;         // 周期性读取定时器

    // 配置参数
    uint8_t scroll_limit;             // 滚动阈值（默认10像素）
    uint8_t scroll_throw;              // 滚动衰减（默认10%）
    uint8_t gesture_min_velocity;      // 手势最小速度
    uint8_t gesture_min_distance;      // 手势最小距离
    uint16_t long_press_time;         // 长按时间（默认400ms）
    uint16_t long_press_repeat_time;  // 长按重复间隔（默认100ms）

    // 指针/按钮数据
    struct {
        lv_point_t act_point;         // 当前点
        lv_point_t last_point;        // 上一个点
        lv_point_t last_raw_point;    // 原始读取的点
        lv_point_t vect;              // 移动向量
        lv_point_t vect_hist[8];      // 向量历史
        lv_point_t scroll_sum;        // 滚动累计
        lv_obj_t * act_obj;           // 当前按下的对象
        lv_obj_t * scroll_obj;        // 当前滚动的对象
        // ... 更多字段
    } pointer;

    // 键盘数据
    struct {
        lv_indev_state_t last_state;
        uint32_t last_key;
    } keypad;

    lv_obj_t * cursor;                 // 指针类型的鼠标光标
    lv_group_t * group;               // 键盘组
};
```

---

## 4. 输入流程

### 4.1 整体数据流

```
硬件设备 (evdev / SDL / ...)
    ↓
输入驱动 (lv_evdev.c / lv_sdl.c / ...)
    ↓ 填充 lv_indev_data_t
输入设备核心 (lv_indev.c)
    ↓ lv_indev_read()
类型分发 (indev_pointer_proc / indev_keypad_proc / ...)
    ↓
对象查找 (pointer_search_obj)
    ↓
事件处理 (LV_EVENT_PRESSED / LV_EVENT_CLICKED / ...)
    ↓
UI对象处理 (lv_obj.c)
```

### 4.2 定时器模式读取流程

```c
// lv_indev_read_timer_cb - 定时器回调
lv_timer_create(lv_indev_read_timer_cb, LV_DEF_REFR_PERIOD, indev);  // 创建定时器
    ↓
lv_indev_read_timer_cb(timer)
    ↓
lv_indev_read(indev)  // 主处理函数
    ↓
indev_read_core()     // 调用驱动读取数据
    ↓
switch(indev->type)   // 类型分发
    ├── LV_INDEV_TYPE_POINTER → indev_pointer_proc()
    ├── LV_INDEV_TYPE_KEYPAD  → indev_keypad_proc()
    ├── LV_INDEV_TYPE_ENCODER → indev_encoder_proc()
    └── LV_INDEV_TYPE_BUTTON  → indev_button_proc()
```

### 4.3 指针类型处理流程 (`indev_pointer_proc`)

```
接收 lv_indev_data_t { point, state, ... }
    ↓
1. 保存原始坐标
   indev->pointer.last_raw_point = data->point
    ↓
2. 坐标校准（如需要）
   lv_display_rotate_point()
    ↓
3. 坐标合理性检查
    ↓
4. 更新光标位置
   lv_obj_set_pos(cursor, point)
    ↓
5. 处理指针差异
   indev_proc_pointer_diff()
    ↓
6. 状态分发
   ├── PRESSED  → indev_proc_press()
   └── RELEASED → indev_proc_release()
```

### 4.4 按下处理 (`indev_proc_press`)

```
indev_proc_press()
    ↓
1. 查找按下的对象
   pointer_search_obj(disp, &act_point)
    ↓
2. 更新活动对象
   indev->pointer.act_obj = 找到的对象
    ↓
3. 发送事件
   ├── LV_EVENT_HOVER_OVER   (新对象悬停)
   ├── LV_EVENT_PRESSED      (按下)
   └── LV_EVENT_PRESS_LOST   (从旧对象丢失)
    ↓
4. 处理滚动
   lv_indev_scroll_handler()
    ↓
5. 处理手势
   indev_gesture()
    ↓
6. 处理长按
   if (按住时间 >= long_press_time)
       → LV_EVENT_LONG_PRESSED
       → LV_EVENT_LONG_PRESSED_REPEAT
```

### 4.5 释放处理 (`indev_proc_release`)

```
indev_proc_release()
    ↓
1. 处理悬停状态变化
    ↓
2. 发送释放事件
   LV_EVENT_RELEASED
    ↓
3. 发送点击事件
   ├── LV_EVENT_SHORT_CLICKED
   ├── LV_EVENT_SINGLE_CLICKED (第1次点击)
   ├── LV_EVENT_DOUBLE_CLICKED (第2次点击)
   └── LV_EVENT_TRIPLE_CLICKED (第3次点击)
    ↓
4. 处理滚动惯性
   lv_indev_scroll_throw_handler()
```

---

## 5. 键盘处理流程

### 5.1 `indev_keypad_proc`

```
按键事件
    ↓
1. 跳过等待释放状态
    ↓
2. 按键重映射（可选）
   indev->key_remap_cb()
    ↓
3. 更新状态
   indev->keypad.last_key = data->key
   indev->keypad.last_state = data->state
    ↓
4. 发送 LV_EVENT_KEY 事件
    ↓
5. 处理组焦点
   ├── LV_KEY_NEXT  → lv_group_focus_next()
   ├── LV_KEY_PREV  → lv_group_focus_prev()
   └── LV_KEY_ENTER → lv_group_send_data()
    ↓
6. 长按处理
   LV_EVENT_LONG_PRESSED
   LV_EVENT_LONG_PRESSED_REPEAT
```

---

## 6. 编码器处理流程

### 6.1 `indev_encoder_proc`

编码器有三种输入：
1. **旋转**：Left / Right
2. **按钮**：Enter
3. **长按**：切换编辑模式

```
编码器事件
    ↓
1. 处理按钮状态
    ↓
2. 编辑模式 vs 导航模式
   ├── 编辑模式：旋转 → 发送 LV_KEY_LEFT/RIGHT
   └── 导航模式：旋转 → 切换焦点对象
    ↓
3. 按钮处理
   ├── 短按 → LV_EVENT_CLICKED
   ├── 长按 → 切换编辑/导航模式
   └── 释放 → 发送 ENTER 键
```

---

## 7. 滚动系统

### 7.1 滚动查找 (`lv_indev_scroll_handler`)

```c
void lv_indev_scroll_handler(lv_indev_t * indev)
{
    // 1. 检查滚动方向
    lv_dir_t scroll_dir = lv_indev_get_scroll_dir(indev);

    // 2. 查找可滚动对象
    lv_obj_t * scroll_obj = lv_indev_find_scroll_obj(indev);

    // 3. 执行滚动
    if (scroll_obj) {
        lv_obj_scroll_by(scroll_obj, vect.x, vect.y, true);
    }
}
```

### 7.2 滚动惯性 (`lv_indev_scroll_throw_handler`)

当用户快速滑动后释放，滚动惯性会根据速度向量继续滚动并逐渐减速。

---

## 8. 手势识别

### 8.1 支持的手势类型

```c
typedef enum {
    LV_INDEV_GESTURE_NONE = 0,
    LV_INDEV_GESTURE_PINCH,           // 双指捏合
    LV_INDEV_GESTURE_SWIPE,           // 单指滑动
    LV_INDEV_GESTURE_ROTATE,          // 双指旋转
    LV_INDEV_GESTURE_TWO_FINGERS_SWIPE, // 双指滑动
    LV_INDEV_GESTURE_SCROLL,          // 单指滚动
} lv_indev_gesture_type_t;
```

### 8.2 手势检测流程

```
触摸事件 (EV_ABS / EV_SYN)
    ↓
缓冲多触摸点数据
    ↓
SYN_REPORT 事件
    ↓
更新手势识别器
   lv_indev_gesture_recognizers_update()
    ↓
检测手势类型
   ├── pinch_detect()
   ├── rotation_detect()
   └── two_fingers_swipe_detect()
    ↓
发送 LV_EVENT_GESTURE 事件
```

---

## 9. 事件驱动模式

当 `mode = LV_INDEV_MODE_EVENT` 时：

1. 定时器暂停
2. 只在事件发生时读取输入
3. 适合中断驱动的硬件

```c
void lv_indev_set_mode(lv_indev_t * indev, lv_indev_mode_t mode)
{
    if (mode == LV_INDEV_MODE_EVENT) {
        lv_timer_pause(indev->read_timer);  // 暂停定时器
    } else if (mode == LV_INDEV_MODE_TIMER) {
        lv_timer_resume(indev->read_timer); // 恢复定时器
    }
}
```

---

## 10. 输入设备驱动示例 (evdev)

### 10.1 设备发现

```c
// 使用 inotify 监控 /dev/input 目录
lv_evdev_discovery_start(cb, user_data);
    ↓
扫描 /dev/input/event* 文件
    ↓
为每个设备创建输入设备
    ↓
设备插拔时自动回调
```

### 10.2 事件读取

```c
static void _evdev_read(lv_indev_t * indev, lv_indev_data_t * data)
{
    struct input_event in;
    while (read(fd, &in, sizeof(in)) > 0) {
        switch (in.type) {
            case EV_REL:   // 相对移动 (鼠标)
                if (in.code == REL_X) dsc->root_x += in.value;
                if (in.code == REL_Y) dsc->root_y += in.value;
                break;
            case EV_ABS:   // 绝对坐标 (触摸)
                if (in.code == ABS_X) dsc->root_x = in.value;
                if (in.code == ABS_Y) dsc->root_y = in.value;
                break;
            case EV_KEY:   // 按键事件
                dsc->state = in.value ? PRESSED : RELEASED;
                break;
        }
    }

    // 填充返回数据
    data->point = 校准后的坐标(dsc->root_x, dsc->root_y);
    data->state = dsc->state;
}
```

---

## 11. 常用API

### 11.1 设备管理

```c
// 创建输入设备
lv_indev_t * lv_indev_create(void);

// 删除输入设备
void lv_indev_delete(lv_indev_t * indev);

// 获取下一个输入设备
lv_indev_t * lv_indev_get_next(lv_indev_t * indev);

// 启用/禁用输入设备
void lv_indev_enable(lv_indev_t * indev, bool enable);
```

### 11.2 配置

```c
// 设置设备类型
void lv_indev_set_type(lv_indev_t * indev, lv_indev_type_t indev_type);

// 设置读取回调
void lv_indev_set_read_cb(lv_indev_t * indev, lv_indev_read_cb_t read_cb);

// 设置关联显示
void lv_indev_set_display(lv_indev_t * indev, lv_display_t * disp);

// 设置光标对象
void lv_indev_set_cursor(lv_indev_t * indev, lv_obj_t * cur_obj);

// 设置键盘组
void lv_indev_set_group(lv_indev_t * indev, lv_group_t * group);

// 设置长按时间
void lv_indev_set_long_press_time(lv_indev_t * indev, uint16_t long_press_time);

// 设置滚动阈值
void lv_indev_set_scroll_limit(lv_indev_t * indev, uint8_t scroll_limit);
```

### 11.3 查询

```c
// 获取当前活动的输入设备
lv_indev_t * lv_indev_active(void);

// 获取设备类型
lv_indev_type_t lv_indev_get_type(const lv_indev_t * indev);

// 获取设备状态
lv_indev_state_t lv_indev_get_state(const lv_indev_t * indev);

// 获取按下的坐标
void lv_indev_get_point(const lv_indev_t * indev, lv_point_t * point);

// 获取按下的键
uint32_t lv_indev_get_key(const lv_indev_t * indev);

// 获取滚动方向
lv_dir_t lv_indev_get_scroll_dir(const lv_indev_t * indev);
```

---

## 12. 事件类型总结

| 事件类型 | 触发时机 | 适用设备 |
|---------|---------|---------|
| LV_EVENT_PRESSED | 按下 | 全部 |
| LV_EVENT_PRESSING | 按住移动中 | POINTER, BUTTON |
| LV_EVENT_PRESS_LOST | 按下后滑动到其他对象 | POINTER |
| LV_EVENT_RELEASED | 释放 | 全部 |
| LV_EVENT_SHORT_CLICKED | 短按释放 | 全部 |
| LV_EVENT_CLICKED | 点击 | 全部 |
| LV_EVENT_LONG_PRESSED | 长按触发 | 全部 |
| LV_EVENT_LONG_PRESSED_REPEAT | 长按重复触发 | 全部 |
| LV_EVENT_GESTURE | 手势识别 | POINTER |
| LV_EVENT_KEY | 按键 | KEYPAD, ENCODER |
| LV_EVENT_HOVER_OVER | 悬停进入 | POINTER |
| LV_EVENT_HOVER_LEAVE | 悬停离开 | POINTER |
| LV_EVENT_SCROLL_THROW_BEGIN | 滚动惯性开始 | POINTER |
| LV_EVENT_FOCUSED | 获得焦点 | 全部 |
| LV_EVENT_DEFOCUSED | 失去焦点 | 全部 |
| LV_EVENT_ROTARY | 旋编码器转 | ENCODER |

---

## 13. SIM 环境下的输入系统对接 (SDL 驱动)

### 13.1 架构概述

在 NuttX SIM 模拟器环境下，LVGL 使用 SDL 作为底层窗口和输入系统，实现完整的桌面模拟环境。

```
SDL 事件循环 (SDL_PollEvent)
    ↓
lv_sdl_window.c: sdl_event_handler() (每 5ms 定时器调用)
    ↓ 分发事件到各驱动
    ├── lv_sdl_mouse.c: lv_sdl_mouse_handler()
    ├── lv_sdl_mousewheel.c: lv_sdl_mousewheel_handler()
    └── lv_sdl_keyboard.c: lv_sdl_keyboard_handler()
    ↓ 更新驱动内部状态
    ↓ 调用 lv_indev_read() 触发 LVGL 输入系统
lv_indev.c: lv_indev_read() → 事件处理 → UI 更新
```

### 13.2 SDL 事件循环

SDL 事件处理通过定时器周期性调用，每 5ms 检查一次：

```c
// lv_sdl_window.c
static lv_timer_t * event_handler_timer;

lv_display_t * lv_sdl_window_create(int32_t hor_res, int32_t ver_res)
{
    if(!inited) {
        SDL_Init(SDL_INIT_VIDEO);              // 初始化 SDL 视频系统
        SDL_StartTextInput();                   // 启用文本输入
        event_handler_timer = lv_timer_create(
            sdl_event_handler, 5, NULL);        // 创建 5ms 定时器
        lv_tick_set_cb(SDL_GetTicks);           // 设置时钟回调
        inited = true;
    }
    // ...
}

static void sdl_event_handler(lv_timer_t * t)
{
    SDL_Event event;
    while(SDL_PollEvent(&event)) {              // 轮询所有 SDL 事件
        lv_sdl_mouse_handler(&event);           // 处理鼠标事件
        lv_sdl_keyboard_handler(&event);        // 处理键盘事件
        
        if(event.type == SDL_QUIT) {            // 处理退出事件
            SDL_Quit();
            lv_deinit();
            inited = false;
            exit(0);
        }
    }
}
```

### 13.3 鼠标驱动对接

#### 13.3.1 鼠标驱动创建

```c
// lv_sdl_mouse.c
typedef struct {
    int16_t last_x;          // 最后鼠标 X 坐标
    int16_t last_y;          // 最后鼠标 Y 坐标
    bool left_button_down;   // 左键状态
} lv_sdl_mouse_t;

lv_indev_t * lv_sdl_mouse_create(void)
{
    lv_sdl_mouse_t * dsc = lv_malloc_zeroed(sizeof(lv_sdl_mouse_t));
    lv_indev_t * indev = lv_indev_create();
    
    lv_indev_set_type(indev, LV_INDEV_TYPE_POINTER);     // 设置类型
    lv_indev_set_read_cb(indev, sdl_mouse_read);          // 设置读取回调
    lv_indev_set_driver_data(indev, dsc);                 // 设置驱动私有数据
    
    lv_indev_set_mode(indev, LV_INDEV_MODE_EVENT);       // 事件驱动模式
    lv_indev_add_event_cb(indev, release_indev_cb,       // 释放回调
                          LV_EVENT_DELETE, indev);
    
    return indev;
}
```

#### 13.3.2 鼠标读取回调

```c
static void sdl_mouse_read(lv_indev_t * indev, lv_indev_data_t * data)
{
    lv_sdl_mouse_t * dsc = lv_indev_get_driver_data(indev);
    
    data->point.x = dsc->last_x;                                    // 坐标
    data->point.y = dsc->last_y;
    data->state = dsc->left_button_down ? LV_INDEV_STATE_PRESSED 
                                        : LV_INDEV_STATE_RELEASED; // 状态
}
```

#### 13.3.3 鼠标事件处理

```c
void lv_sdl_mouse_handler(SDL_Event * event)
{
    switch(event->type) {
        case SDL_MOUSEBUTTONUP:
        case SDL_MOUSEBUTTONDOWN:
            win_id = event->button.windowID;
            break;
        case SDL_MOUSEMOTION:
            win_id = event->motion.windowID;
            break;
        case SDL_FINGERUP:       // 也处理触摸事件（映射为鼠标）
        case SDL_FINGERDOWN:
        case SDL_FINGERMOTION:
            // ...
    }
    
    // 查找对应的输入设备
    lv_indev_t * indev = lv_indev_get_next(NULL);
    while(indev) {
        if(lv_indev_get_type(indev) == LV_INDEV_TYPE_POINTER) {
            break;
        }
        indev = lv_indev_get_next(indev);
    }
    
    lv_sdl_mouse_t * dsc = lv_indev_get_driver_data(indev);
    int32_t hor_res = lv_display_get_horizontal_resolution(disp);
    int32_t ver_res = lv_display_get_vertical_resolution(disp);
    uint8_t zoom = lv_sdl_window_get_zoom(disp);
    
    switch(event->type) {
        case SDL_MOUSEBUTTONDOWN:
            if(event->button.button == SDL_BUTTON_LEFT) {
                dsc->left_button_down = true;
                dsc->last_x = event->motion.x / zoom;
                dsc->last_y = event->motion.y / zoom;
            }
            break;
        case SDL_MOUSEBUTTONUP:
            dsc->left_button_down = false;
            break;
        case SDL_MOUSEMOTION:
            dsc->last_x = event->motion.x / zoom;
            dsc->last_y = event->motion.y / zoom;
            break;
        case SDL_FINGERDOWN:
            dsc->left_button_down = true;
            dsc->last_x = (int16_t)((float)hor_res * event->tfinger.x / zoom);
            dsc->last_y = (int16_t)((float)ver_res * event->tfinger.y / zoom);
            break;
    }
    
    lv_indev_read(indev);  // 关键：触发 LVGL 输入处理
}
```

### 13.4 键盘驱动对接

#### 13.4.1 键盘驱动创建

```c
// lv_sdl_keyboard.c
typedef struct {
    char buf[KEYBOARD_BUFFER_SIZE];  // 按键缓冲区 (32字节)
    bool dummy_read;                 // 虚拟读取标记（用于释放状态）
} lv_sdl_keyboard_t;

lv_indev_t * lv_sdl_keyboard_create(void)
{
    lv_sdl_keyboard_t * dsc = lv_malloc_zeroed(sizeof(lv_sdl_keyboard_t));
    lv_indev_t * indev = lv_indev_create();
    
    lv_indev_set_type(indev, LV_INDEV_TYPE_KEYPAD);
    lv_indev_set_read_cb(indev, sdl_keyboard_read);
    lv_indev_set_driver_data(indev, dsc);
    lv_indev_set_mode(indev, LV_INDEV_MODE_EVENT);  // 事件驱动模式
    
    return indev;
}
```

#### 13.4.2 键盘读取回调

```c
static void sdl_keyboard_read(lv_indev_t * indev, lv_indev_data_t * data)
{
    lv_sdl_keyboard_t * dev = lv_indev_get_driver_data(indev);
    const size_t len = lv_strlen(dev->buf);
    
    // 发送释放状态（处理上一次按键）
    if(dev->dummy_read) {
        dev->dummy_read = false;
        data->state = LV_INDEV_STATE_RELEASED;
    }
    // 发送按下的字符
    else if(len > 0) {
        dev->dummy_read = true;
        data->state = LV_INDEV_STATE_PRESSED;
        data->key = dev->buf[0];          // 取出缓冲区第一个字符
        lv_memmove(dev->buf, dev->buf + 1, len); // 移除已处理字符
    }
}
```

#### 13.4.3 键盘事件处理

```c
void lv_sdl_keyboard_handler(SDL_Event * event)
{
    switch(event->type) {
        case SDL_KEYDOWN:
            // 控制键映射
            const uint32_t ctrl_key = keycode_to_ctrl_key(event->key.keysym.sym);
            if(ctrl_key != '\0') {
                dsc->buf[len] = ctrl_key;  // 加入缓冲区
            }
            break;
        case SDL_TEXTINPUT:
            // 文本输入（字符键）
            strcat(dsc->buf, event->text.text);
            break;
    }
    
    // 处理缓冲区中的所有按键
    size_t len = lv_strlen(dsc->buf);
    while(len) {
        lv_indev_read(indev);      // 触发按下
        lv_indev_read(indev);      // 触发释放（dummy read）
        len--;
    }
}

// SDL 键码 → LVGL 控制键映射
static uint32_t keycode_to_ctrl_key(SDL_Keycode sdl_key)
{
    switch(sdl_key) {
        case SDLK_RIGHT:     return LV_KEY_RIGHT;
        case SDLK_LEFT:      return LV_KEY_LEFT;
        case SDLK_UP:        return LV_KEY_UP;
        case SDLK_DOWN:      return LV_KEY_DOWN;
        case SDLK_ESCAPE:    return LV_KEY_ESC;
        case SDLK_BACKSPACE: return LV_KEY_BACKSPACE;
        case SDLK_DELETE:    return LV_KEY_DEL;
        case SDLK_ENTER:     return LV_KEY_ENTER;
        case SDLK_TAB:       return LV_KEY_NEXT;
        case SDLK_PAGEUP:    return LV_KEY_PREV;
        case SDLK_HOME:      return LV_KEY_HOME;
        case SDLK_END:       return LV_KEY_END;
        default:             return '\0';
    }
}
```

### 13.5 关键设计模式

#### 13.5.1 事件驱动模式

所有 SDL 输入设备都使用 `LV_INDEV_MODE_EVENT` 模式：
- 定时器模式：周期性调用读取回调
- 事件模式：仅在 `lv_indev_read()` 被调用时处理

```
SDL_PollEvent() → lv_sdl_xxx_handler() → 更新驱动状态 → lv_indev_read()
```

#### 13.5.2 驱动私有数据

每个输入设备都有私有数据结构体，通过 `lv_indev_set_driver_data()` 关联：

```c
// 设置
lv_indev_set_driver_data(indev, dsc);

// 获取
lv_sdl_mouse_t * dsc = lv_indev_get_driver_data(indev);
```

#### 13.5.3 窗口-显示-输入关联

```
SDL_Window → lv_display_t → lv_indev_t
    ↓              ↓            ↓
  windowID     display       read_cb
               driver_data   driver_data
```

### 13.6 SIM 环境完整流程图

```
main()
    ↓
lv_init()
    ↓
lv_sdl_window_create(800, 480)  ← 创建 SDL 窗口
    ├── SDL_Init(SDL_INIT_VIDEO)
    ├── SDL_CreateWindow()
    ├── lv_timer_create(sdl_event_handler, 5, NULL)
    └── lv_sdl_mouse_create()   ← 创建鼠标输入设备
    └── lv_sdl_keyboard_create() ← 创建键盘输入设备
    
主循环:
    lv_timer_handler()  ← 每 1ms 调用
        ↓
    sdl_event_handler()  ← 每 5ms 调用
        ↓
    while(SDL_PollEvent(&event))
        ↓
    lv_sdl_mouse_handler(&event)
        ├── 更新 dsc->last_x, dsc->last_y
        ├── 更新 dsc->left_button_down
        └── lv_indev_read()  ← 触发 LVGL 输入处理
            ↓
        sdl_mouse_read()  ← 读取回调
            ├── data->point.x = dsc->last_x
            ├── data->point.y = dsc->last_y
            └── data->state = dsc->left_button_down ? PRESSED : RELEASED
            ↓
        lv_indev.c: 类型分发 → 对象查找 → 事件处理 → UI 更新
```

---

## 14. 代码位置索引

| 功能 | 文件 | 主要函数 |
|-----|------|---------|
| 输入设备管理 | `lv_indev.c` | `lv_indev_create()`, `lv_indev_read()` |
| 指针处理 | `lv_indev.c` | `indev_pointer_proc()`, `indev_proc_press()`, `indev_proc_release()` |
| 键盘处理 | `lv_indev.c` | `indev_keypad_proc()` |
| 编码器处理 | `lv_indev.c` | `indev_encoder_proc()` |
| 按钮处理 | `lv_indev.c` | `indev_button_proc()` |
| 滚动处理 | `lv_indev_scroll.c` | `lv_indev_scroll_handler()`, `lv_indev_scroll_throw_handler()` |
| 手势识别 | `lv_indev_gesture.c` | `lv_indev_gesture_init()`, `lv_indev_gesture_recognizers_update()` |
| evdev驱动 | `lv_evdev.c` | `_evdev_read()`, `lv_evdev_create()` |
| 对象查找 | `lv_indev.c` | `pointer_search_obj()`, `lv_indev_search_obj()` |

---

*文档版本: 1.0*
*基于 LVGL 源码分析生成*
