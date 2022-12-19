# -*- coding: utf-8 -*-
# Copyright (c) 2022, Harry Huang
# @ BSD 3-Clause License
'''
BSD 3-Clause License

Copyright (c) 2022, Harry Huang
All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this
   list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.

3. Neither the name of the copyright holder nor the names of its
   contributors may be used to endorse or promote products derived from
   this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
'''
import os, time
from src_py.osTool import *
from src_py.colorTool import *
from src_py import ResolveAB as AU_Rs
from src_py import CombineRGBwithA as AU_Cb
'''
ArkUnpacker主程序
'''
AU_ver='v2.0'
AU_i18n='zh-CN'
MAX_THS=21


def input_allow(msg:str,allow:list,excpt:str):
    '''
    #### 获取合规的键盘命令输入
    :param msg:   提示信息;
    :param allow: 包含了合规的输入的列表;
    :param excpt: 输入不合规时的提示信息;
    :returns:     (str) 一个合规的输入;
    '''
    inpt = input(msg)
    while not (inpt in allow):
        inpt = input(excpt)
    return inpt

def input_path(msg:str,excpt:str):
    '''
    #### 获取合规的目录路径输入
    :param msg:   提示信息;
    :param excpt: 输入目录不存在时的提示信息;
    :returns:     (str) 一个合规的输入;
    '''
    inpt = os.path.normpath(input(msg))
    while not os.path.isdir(inpt):
            inpt = os.path.normpath(input(excpt))
    return inpt

def get_dirlist(ignore:list=[]):
    '''
    #### 获取当前目录下的第一级子目录的列表
    :param ignore: 可选，忽略名单，精确匹配;
    :returns:      (list) 子目录的列表;
    '''
    filelist = []
    for i in os.listdir():
        if os.path.isdir(i) and os.path.basename(i) not in ignore:
            filelist.append(i)
    return filelist

def run_quickaccess():
    '''
    #### 启动一键执行模式
    :returns: (none);
    '''
    os.system('title ArkUnpacker - Processing')
    destdir = f'Unpacked'
    ignore = [".vscode","__pycache__",".git"]
    ###
    time.sleep(1)
    AU_Rs.main('.',destdir)
    ###
    time.sleep(1)
    AU_Cb.main(destdir,f'Combined_{int(time.time())}')

if __name__ == '__main__':
        os.system('title ArkUnpacker')
        run_quickaccess()
        exit()