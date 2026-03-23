// mfind GUI - Frontend JavaScript

// State
let isSearching = false;
let isBuilding = false;

// DOM elements
const searchInput = document.getElementById('searchInput');
const searchBtn = document.getElementById('searchBtn');
const limitCheck = document.getElementById('limitCheck');
const limitInput = document.getElementById('limitInput');
const statusText = document.getElementById('statusText');
const statsText = document.getElementById('statsText');
const resultCount = document.getElementById('resultCount');
const resultsList = document.getElementById('resultsList');
const buildIndexBtn = document.getElementById('buildIndexBtn');
const indexPath = document.getElementById('indexPath');
const buildStatus = document.getElementById('buildStatus');

// Initialize
document.addEventListener('DOMContentLoaded', () => {
    setupEventListeners();
    loadStats();
});

function setupEventListeners() {
    // Search button click
    searchBtn.addEventListener('click', performSearch);

    // Enter key in search input
    searchInput.addEventListener('keypress', (e) => {
        if (e.key === 'Enter') {
            performSearch();
        }
    });

    // Build index button
    buildIndexBtn.addEventListener('click', buildIndex);

    // Real-time search on input (debounced)
    let searchTimeout;
    searchInput.addEventListener('input', () => {
        clearTimeout(searchTimeout);
        searchTimeout = setTimeout(() => {
            if (searchInput.value.length >= 2) {
                performSearch();
            }
        }, 300);
    });
}

// Search function
async function performSearch() {
    const pattern = searchInput.value.trim();
    if (!pattern) {
        setStatus('请输入搜索关键词');
        return;
    }

    if (isSearching) {
        return;
    }

    isSearching = true;
    setSearchBtnLoading(true);
    setStatus('搜索中...');
    resultsList.innerHTML = '<div class="loading"><span class="spinner"></span> 搜索中...</div>';

    try {
        const limit = limitCheck.checked ? parseInt(limitInput.value) : undefined;

        // Call Tauri command
        const result = await window.__TAURI__.core.invoke('search', {
            pattern,
            limit
        });

        displayResults(result);
        setStatus(`搜索完成，耗时 ${result.query_time_ms.toFixed(2)}ms`);
    } catch (error) {
        setStatus(`搜索失败：${error}`);
        resultsList.innerHTML = '<div class="empty-state">搜索出错</div>';
    } finally {
        isSearching = false;
        setSearchBtnLoading(false);
    }
}

// Display search results
function displayResults(response) {
    const { results, total } = response;

    resultCount.textContent = `${total} 个结果`;

    if (results.length === 0) {
        resultsList.innerHTML = '<div class="empty-state">未找到匹配的结果</div>';
        return;
    }

    resultsList.innerHTML = results.map(item => `
        <div class="result-item" onclick="copyPath('${escapeHtml(item.path)}')">
            <span class="result-icon">${item.is_dir ? '📁' : '📄'}</span>
            <span class="result-name">${escapeHtml(item.path)}</span>
            ${item.is_dir ? '<span class="result-type">目录</span>' : ''}
        </div>
    `).join('');
}

// Load index stats
async function loadStats() {
    try {
        const stats = await window.__TAURI__.core.invoke('get_stats');
        statsText.textContent = `索引：${formatNumber(stats.total_files)} 文件，${formatNumber(stats.total_dirs)} 目录`;
    } catch (error) {
        statsText.textContent = '索引统计不可用';
    }
}

// Build index
async function buildIndex() {
    const path = indexPath.value.trim();
    if (!path) {
        setBuildStatus('请输入路径', 'error');
        return;
    }

    if (isBuilding) {
        return;
    }

    isBuilding = true;
    buildIndexBtn.disabled = true;
    buildIndexBtn.textContent = '构建中...';
    setBuildStatus('正在构建索引，请稍候...', '');

    try {
        const result = await window.__TAURI__.core.invoke('build_index', {
            paths: [path]
        });

        setBuildStatus(
            `索引构建完成！${result.total_files} 个文件，${result.total_dirs} 个目录，耗时 ${result.build_time_ms.toFixed(2)}ms`,
            'success'
        );

        // Refresh stats
        loadStats();
    } catch (error) {
        setBuildStatus(`构建失败：${error}`, 'error');
    } finally {
        isBuilding = false;
        buildIndexBtn.disabled = false;
        buildIndexBtn.textContent = '构建索引';
    }
}

// Utility functions
function setStatus(message) {
    statusText.textContent = message;
}

function setBuildStatus(message, type) {
    buildStatus.textContent = message;
    buildStatus.className = 'build-status ' + type;
}

function setSearchBtnLoading(loading) {
    if (loading) {
        searchBtn.innerHTML = '<span class="spinner"></span>';
    } else {
        searchBtn.textContent = '搜索';
    }
}

function formatNumber(num) {
    if (num >= 1000000) {
        return (num / 1000000).toFixed(1) + 'M';
    }
    if (num >= 1000) {
        return (num / 1000).toFixed(1) + 'K';
    }
    return num.toString();
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML.replace(/'/g, "&#39;");
}

// Copy path to clipboard
function copyPath(path) {
    navigator.clipboard.writeText(path).then(() => {
        setStatus('路径已复制到剪贴板');
    }).catch(() => {
        setStatus('复制失败');
    });
}
