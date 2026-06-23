import { useState, useEffect } from 'react'
import { getScripts, runScript, type Script, type ScriptResult } from '../lib/api'

interface ScriptWithState extends Script {
  _paramMode: 'day' | 'month'
  _dayParam: string
  _monthParam: string
  _running: boolean
  _result: (ScriptResult & { _msg: string }) | null
}

function getYesterday() {
  const d = new Date()
  d.setDate(d.getDate() - 1)
  return d.toISOString().slice(0, 10)
}

function getCurrentMonth() {
  const d = new Date()
  return d.toISOString().slice(0, 7)
}

export default function ScriptRunner() {
  const [scripts, setScripts] = useState<ScriptWithState[]>([])

  useEffect(() => { loadScripts() }, [])

  async function loadScripts() {
    const list = await getScripts()
    const dayDefault = getYesterday()
    const monthDefault = getCurrentMonth()
    const sortedList = [...list].sort((a, b) => (a.order || 0) - (b.order || 0))
    setScripts(sortedList.map(s => ({
      ...s,
      _paramMode: s.param_type === 'both' ? 'day' : s.param_type as 'day' | 'month',
      _dayParam: (s.param_type === 'day' || s.param_type === 'both') ? dayDefault : '',
      _monthParam: (s.param_type === 'month' || s.param_type === 'both') ? monthDefault : '',
      _running: false,
      _result: null,
    })))
  }

  function getParamTypeLabel(type: string) {
    if (type === 'day') return '日报'
    if (type === 'month') return '月报'
    return '日报/月报'
  }

  function getEffectiveParam(script: ScriptWithState) {
    if (script.param_type === 'day' || (script.param_type === 'both' && script._paramMode === 'day')) {
      return script._dayParam?.replace(/-/g, '') || ''
    }
    return script._monthParam?.replace(/-/g, '') || ''
  }

  function canRun(script: ScriptWithState) {
    if (script._running) return false
    const param = getEffectiveParam(script)
    if (!param) return false
    if (script._paramMode === 'day' || script.param_type === 'day') return param.length === 8
    return param.length === 6
  }

  async function handleRun(script: ScriptWithState) {
    const param = getEffectiveParam(script)
    if (!param) return

    // 标记为运行中
    setScripts(prev => prev.map(s =>
      s.id === script.id ? { ...s, _running: true, _result: null } : s
    ))

    try {
      // runScript 现在是 async 的，不会阻塞 UI
      const result = await runScript(script.id, param)
      const msg = result.success ? '执行成功' : '执行失败: ' + (result.stderr || '未知错误')
      setScripts(prev => prev.map(s =>
        s.id === script.id ? { ...s, _running: false, _result: { ...result, _msg: msg } } : s
      ))
    } catch (e: any) {
      setScripts(prev => prev.map(s =>
        s.id === script.id ? { ...s, _running: false, _result: { success: false, stdout: '', stderr: '', returncode: -1, _msg: '执行异常: ' + e } } : s
      ))
    }
  }

  function updateScriptState(id: string, updates: Partial<ScriptWithState>) {
    setScripts(prev => prev.map(s => s.id === id ? { ...s, ...updates } : s))
  }

  return (
    <div className="space-y-3 pb-20">
      <h2 className="text-base font-medium text-gray-900">脚本执行</h2>

      {scripts.length === 0 ? (
        <div className="text-center py-12 text-gray-400 text-sm">暂无脚本，请在脚本管理页面添加</div>
      ) : (
        <div className="space-y-2">
          {scripts.map((script) => (
            <div key={script.id} className="bg-white rounded-lg border border-gray-200 p-3">
              <div className="flex items-center space-x-3">
                <div className="w-28 shrink-0">
                  <h3 className="text-sm font-medium text-gray-900 truncate" title={script.name}>{script.name}</h3>
                </div>

                <div className="w-16 shrink-0">
                  <span className="text-sm text-gray-500">{getParamTypeLabel(script.param_type)}</span>
                </div>

                <div className="w-40 shrink-0">
                  {(script.param_type === 'day' || (script.param_type === 'both' && script._paramMode === 'day')) && (
                    <input type="date" value={script._dayParam}
                      onChange={e => updateScriptState(script.id, { _dayParam: e.target.value })}
                      disabled={script._running}
                      className="w-full px-2 py-1 border border-gray-300 rounded text-sm focus:ring-1 focus:ring-blue-500 focus:border-blue-500 disabled:opacity-50" />
                  )}
                  {(script.param_type === 'month' || (script.param_type === 'both' && script._paramMode === 'month')) && (
                    <input type="month" value={script._monthParam}
                      onChange={e => updateScriptState(script.id, { _monthParam: e.target.value })}
                      disabled={script._running}
                      className="w-full px-2 py-1 border border-gray-300 rounded text-sm focus:ring-1 focus:ring-blue-500 focus:border-blue-500 disabled:opacity-50" />
                  )}
                </div>

                {script.param_type === 'both' && (
                  <div className="w-24 shrink-0">
                    <div className="flex rounded border border-gray-300 overflow-hidden">
                      <button
                        disabled={script._running}
                        onClick={() => updateScriptState(script.id, { _paramMode: 'day' })}
                        className={`flex-1 px-2 py-1 text-xs font-medium transition-colors disabled:opacity-50 ${
                          script._paramMode === 'day' ? 'bg-blue-600 text-white' : 'bg-white text-gray-600'
                        }`}>
                        日报
                      </button>
                      <button
                        disabled={script._running}
                        onClick={() => updateScriptState(script.id, { _paramMode: 'month' })}
                        className={`flex-1 px-2 py-1 text-xs font-medium transition-colors disabled:opacity-50 ${
                          script._paramMode === 'month' ? 'bg-blue-600 text-white' : 'bg-white text-gray-600'
                        }`}>
                        月报
                      </button>
                    </div>
                  </div>
                )}

                <div className="flex-1" />

                <div className="w-24 shrink-0 text-right">
                  <button
                    onClick={() => handleRun(script)}
                    disabled={!canRun(script)}
                    className={`px-3 py-1 text-sm font-medium rounded transition-colors disabled:opacity-40 disabled:cursor-not-allowed ${
                      script._running
                        ? 'bg-yellow-100 text-yellow-700 cursor-wait'
                        : 'bg-blue-600 text-white hover:bg-blue-700'
                    }`}>
                    {script._running ? '执行中...' : '执行'}
                  </button>
                </div>
              </div>

              {script._result && (
                <div className={`mt-2 px-3 py-2 rounded text-xs ${script._result.success ? 'bg-green-50 text-green-700' : 'bg-red-50 text-red-700'}`}>
                  {script._result._msg}
                  {script._result.stdout && (
                    <pre className="mt-1 text-xs whitespace-pre-wrap max-h-32 overflow-y-auto">{script._result.stdout}</pre>
                  )}
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
