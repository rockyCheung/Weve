import os

COPYRIGHT_HOLDER = "Beijing Xibei Technology Co., Ltd."
AUTHOR = "Zhang Penghong"
YEAR = "2026"

# Corrected headers (single dot after Ltd)
PYTHON_HEADER = f"""# -*- coding: utf-8 -*-
#
# Copyright (c) {YEAR} {COPYRIGHT_HOLDER} All Rights Reserved.
# Author: {AUTHOR}
#
# This software and associated documentation files (the "Software") are the
# proprietary property of {COPYRIGHT_HOLDER} No part of this 
# software may be copied, modified, or distributed without the express 
# written permission of {COPYRIGHT_HOLDER}
#
# Unauthorized copying of this file, via any medium, is strictly prohibited.

"""

JS_HEADER = f"""/**
 * Copyright (c) {YEAR} {COPYRIGHT_HOLDER} All Rights Reserved.
 * Author: {AUTHOR}
 *
 * This software and associated documentation files (the "Software") are the
 * proprietary property of {COPYRIGHT_HOLDER} No part of this 
 * software may be copied, modified, or distributed without the express 
 * written permission of {COPYRIGHT_HOLDER}
 *
 * Unauthorized copying of this file, via any medium, is strictly prohibited.
 */

"""

TARGET_DIRS = [
    "src",
    "src-tauri"
]

EXTENSIONS = {
    ".py": PYTHON_HEADER,
    ".js": JS_HEADER,
    ".jsx": JS_HEADER,
    ".ts": JS_HEADER,
    ".tsx": JS_HEADER,
    ".rs": JS_HEADER
}

def apply_header(file_path, header):
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # Check for the faulty "Ltd.." version and replace if found, or skip if correct
    faulty_str = "Beijing Xibei Technology Co., Ltd.."
    correct_str = "Beijing Xibei Technology Co., Ltd."
    
    if faulty_str in content:
        print(f"Fixing typo in: {file_path}")
        new_content = content.replace(faulty_str, correct_str)
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(new_content)
        return

    if correct_str in content:
        print(f"Skipping (already has correct header): {file_path}")
        return

    print(f"Applying header to: {file_path}")
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(header + content)

def main():
    base_path = os.getcwd()
    for target in TARGET_DIRS:
        full_path = os.path.join(base_path, target)
        if not os.path.exists(full_path):
            print(f"Directory not found: {full_path}")
            continue
            
        for root, dirs, files in os.walk(full_path):
            dirs[:] = [d for d in dirs if not d.startswith('.')]
            for file in files:
                ext = os.path.splitext(file)[1].lower()
                if ext in EXTENSIONS:
                    file_path = os.path.join(root, file)
                    apply_header(file_path, EXTENSIONS[ext])

if __name__ == "__main__":
    main()
