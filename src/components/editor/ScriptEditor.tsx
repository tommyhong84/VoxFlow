import { useState } from 'react';
import { Sparkles, Save, Plus } from 'lucide-react';
import { useScriptStore } from '../../store/scriptStore';
import ScriptLineComponent from './ScriptLine';
import { Button } from '../ui/button';
import { Card, CardContent } from '../ui/card';
import { Label } from '../ui/label';

export default function ScriptEditor() {
    const { lines, isGenerating, isDirty, streamingText, generateScript, saveScript, addLine } =
        useScriptStore();
    const [outline, setOutline] = useState('');

    const handleGenerate = () => {
        if (!outline.trim() || isGenerating) return;
        generateScript(outline.trim());
    };

    return (
        <div className="mx-auto max-w-4xl px-6 py-6 space-y-4">
            <Card>
                <CardContent className="space-y-3">
                    <Label>大纲输入</Label>
                    <textarea
                        className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 outline-none resize-y min-h-[80px] dark:bg-input/30"
                        placeholder="输入有声书大纲，AI 将为你生成剧本..."
                        value={outline}
                        onChange={(e) => setOutline(e.target.value)}
                        disabled={isGenerating}
                    />
                    <div className="flex gap-2">
                        <Button onClick={handleGenerate} disabled={isGenerating || !outline.trim()}>
                            <Sparkles className="h-4 w-4" />
                            {isGenerating ? '生成中...' : '生成剧本'}
                        </Button>
                        {isDirty && (
                            <Button variant="outline" onClick={() => saveScript()}>
                                <Save className="h-4 w-4" /> 保存剧本
                            </Button>
                        )}
                    </div>
                </CardContent>
            </Card>

            {isGenerating && streamingText && (
                <Card className="border-purple-200 dark:border-purple-800 bg-purple-50 dark:bg-purple-900/20">
                    <CardContent>
                        <p className="text-sm text-purple-700 dark:text-purple-300 font-medium mb-2">AI 正在生成...</p>
                        <pre className="text-sm whitespace-pre-wrap">{streamingText}<span className="animate-pulse">▌</span></pre>
                    </CardContent>
                </Card>
            )}

            {lines.length > 0 && (
                <div className="space-y-2">
                    {lines.map((line, index) => (
                        <ScriptLineComponent key={line.id} line={line} index={index} />
                    ))}
                    <Button variant="outline" className="w-full border-dashed" onClick={() => addLine(lines.length - 1)}>
                        <Plus className="h-4 w-4" /> 添加新行
                    </Button>
                </div>
            )}

            {lines.length === 0 && !isGenerating && (
                <p className="text-center text-muted-foreground py-16">输入大纲生成剧本，或手动添加剧本行</p>
            )}

            {lines.length === 0 && !isGenerating && (
                <div className="flex justify-center">
                    <Button variant="outline" className="border-dashed" onClick={() => addLine(-1)}>
                        <Plus className="h-4 w-4" /> 添加第一行
                    </Button>
                </div>
            )}
        </div>
    );
}
