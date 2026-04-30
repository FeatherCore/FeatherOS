# ##############################################################################
# apps/examples/lvgl_window10/CMakeLists.txt
# ##############################################################################

if(CONFIG_EXAMPLES_LVGL_WINDOW10)
  file(GLOB LVGL_WINDOW10_UI_SRCS
    ui/*.c
    ui/components/*.c
    ui/images/*.c
    ui/screens/*.c)

  nuttx_add_application(
    NAME
    lvgl_window10
    PRIORITY
    ${CONFIG_EXAMPLES_LVGL_WINDOW10_PRIORITY}
    STACKSIZE
    ${CONFIG_EXAMPLES_LVGL_WINDOW10_STACKSIZE}
    MODULE
    ${CONFIG_EXAMPLES_LVGL_WINDOW10}
    DEPENDS
    lvgl
    INCLUDES
    ${NUTTX_APPS_DIR}/graphics/lvgl/lvgl
    ${NUTTX_APPS_DIR}/graphics/lvgl/lvgl/src
    ${CMAKE_CURRENT_LIST_DIR}/ui
    SRCS
    lvgl_window10.c
    ${LVGL_WINDOW10_UI_SRCS})
endif()
