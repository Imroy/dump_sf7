cmake_minimum_required(VERSION 3.10)
project(dump_sf7)

set (CMAKE_CXX_STANDARD 11)
set(COMMON_FLAGS "-g -Wall -fopenmp -finput-charset=UTF-8 -Wformat -Wformat-security -Werror=format-security -D_FORTIFY_SOURCE=2 -fstack-protector --param ssp-buffer-size=4 -fPIE -pie")

# LTO
set(CMAKE_LD_FLAGS "${CMAKE_LD_FLAGS} ${CMAKE_CXX_FLAGS} -Wl,-z,relro -Wl,-z,now -fPIE -pie -flto")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} ${COMMON_FLAGS} -flto -fuse-linker-plugin")
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} ${COMMON_FLAGS} -flto -fuse-linker-plugin -Wno-non-template-friend")

file(GLOB CORE_FILES
	  *.hh
	  *.tcc
	  *.cc
)

add_executable(dump_sf7
        ${CORE_FILES}
)
#target_include_directories(dump_sf7
#	.
#)
#target_compile_options(dump_sf7
#        PUBLIC
#)
#target_link_libraries(dump_sf7
#)

install(TARGETS dump_sf7
        DESTINATION /usr/local/bin
)
