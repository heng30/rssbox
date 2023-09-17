use crate::config;
use std::collections::HashMap;

pub fn tr(text: &str) -> String {
    if config::ui().language == "cn" {
        return text.to_string();
    }

    let mut items: HashMap<&str, &str> = HashMap::new();
    items.insert("原因", "Reason");
    items.insert("出错！", "Error!");
    items.insert("新建成功！", "New Success!");
    items.insert("新建失败！", "New Failed!");
    items.insert("删除成功！", "Delete Success!");
    items.insert("删除失败！", "Delete Failed!");
    items.insert("复制失败！", "Copy Failed!");
    items.insert("复制成功！", "Copy Success!");
    items.insert("编辑失败！", "Edit Failed!");
    items.insert("不允许删除！", "Not Allow To Delete!");
    items.insert("删除失败！", "Delete Session Failed!");
    items.insert("删除成功！", "Delete Session Success");
    items.insert("重置成功！", "Reset Success!");
    items.insert("已经收藏！", "Already Marked!");
    items.insert("收藏成功！", "Marked Success!");
    items.insert("取消收藏成功！", "Unmarked Success!");
    items.insert("保存失败！", "Save Failed!");
    items.insert("保存成功！", "Save Success!");
    items.insert("收藏失败！", "Favorite Failed!");
    items.insert("收藏成功！", "Favorite Success!");
    items.insert("清空失败！", "Remove All Failed!");
    items.insert("清空成功！", "Remove All Success!");
    items.insert("隐藏程序失败！", "Hide Window Failed!");
    items.insert("清除缓存失败！", "Clean Cache Failed!");
    items.insert("清除缓存成功！", "Clean Cache Success!");
    items.insert("不允许刷新！", "Not Allow Flushing!");
    items.insert("打开链接失败！", "Open URL Failed!");
    items.insert("同步失败！", "Sync Failed!");
    items.insert("同步成功！", "Sync Success!!");
    items.insert("正在重试...", "retrying...");
    items.insert("正在同步...", "Sync...");

    if let Some(txt) = items.get(text) {
        return txt.to_string();
    }

    text.to_string()
}
