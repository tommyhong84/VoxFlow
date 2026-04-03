import { useState, useEffect } from 'react';
import { Download, Music, AlertTriangle, CheckCircle, Loader2 } from 'lucide-react';
import { useProjectStore } from '../../store/projectStore';
import { useScriptStore } from '../../store/scriptStore';
import * as ipc from '../../lib/ipc';
import { Button } from '../ui/button';
import { Input } from '../ui/input';
import { Label } from '../ui/label';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Alert, AlertTitle, AlertDescription } from '../ui/alert';
import { Progress } from '../ui/progress';
import { Slider } from '../ui/slider';
import type { MixProgress } from '../../types';

export default function ExportPanel() {
    const currentProject = useProjectStore((s) => s.currentProject);
    const { lines } = useScriptStore();
    const [bgmPath, setBgmPath] = useState<string | null>(null);
    const [bgmVolume, setBgmVolume] = useState(0.3);
    const [outputPath, setOutputPath] = useState('');
    const [exporting, setExporting] = useState(false);
    const [progress, setProgress] = useState<MixProgress | null>(null);
    const [done, setDone] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const audioFragments = currentProject?.audio_fragments ?? [];
    const coveredLineIds = new Set(audioFragments.map((a) => a.line_id));
    const missingLines = lines.filter((l) => l.text.trim() && !coveredLineIds.has(l.id));

    useEffect(() => {
        if (currentProject) {
            setOutputPath(`${currentProject.project.name}.mp3`);
        }
    }, [currentProject]);

    const handleExport = async () => {
        if (!currentProject || missingLines.length > 0) return;
        setExporting(true);
        setProgress(null);
        setDone(false);
        setError(null);

        const unlisten = await ipc.onMixProgress((p) => setProgress(p));

        try {
            await ipc.exportAudioMix(
                currentProject.project.id,
                outputPath,
                bgmPath,
                bgmVolume,
            );
            setDone(true);
        } catch (e) {
            setError(String(e));
        } finally {
            unlisten();
            setExporting(false);
        }
    };

    const handleBgmImport = () => {
        // In a real implementation, this would use Tauri's file dialog
    };

    return (
        <div className="mx-auto max-w-3xl px-6 py-8 space-y-6">
            <h2 className="text-xl font-bold">导出有声书</h2>

            {/* Missing audio warning */}
            {missingLines.length > 0 && (
                <Alert variant="destructive">
                    <AlertTriangle className="h-4 w-4" />
                    <AlertTitle>{missingLines.length} 行剧本缺少音频</AlertTitle>
                    <AlertDescription>
                        <ul className="mt-1 space-y-0.5">
                            {missingLines.slice(0, 5).map((l) => (
                                <li key={l.id}>第 {l.line_order + 1} 行: {l.text.slice(0, 40)}...</li>
                            ))}
                            {missingLines.length > 5 && <li>...还有 {missingLines.length - 5} 行</li>}
                        </ul>
                        <p className="mt-2">请先为所有剧本行生成语音后再导出</p>
                    </AlertDescription>
                </Alert>
            )}

            {/* BGM config */}
            <Card>
                <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                        <Music className="h-4 w-4" /> 背景音乐 (BGM)
                    </CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                    <div className="flex gap-3 items-center">
                        <Input
                            className="flex-1"
                            placeholder="BGM 文件路径（可选）"
                            value={bgmPath ?? ''}
                            onChange={(e) => setBgmPath(e.target.value || null)}
                        />
                        <Button variant="outline" onClick={handleBgmImport}>浏览</Button>
                    </div>
                    {bgmPath && (
                        <div className="space-y-2">
                            <Label>BGM 音量 ({Math.round(bgmVolume * 100)}%)</Label>
                            <Slider
                                min={0} max={1} step={0.05}
                                value={[bgmVolume]}
                                onValueChange={([v]) => setBgmVolume(v)}
                            />
                        </div>
                    )}
                </CardContent>
            </Card>

            {/* Output path */}
            <Card>
                <CardContent className="space-y-2">
                    <Label>输出文件名</Label>
                    <Input
                        value={outputPath}
                        onChange={(e) => setOutputPath(e.target.value)}
                    />
                </CardContent>
            </Card>

            {/* Progress */}
            {exporting && progress && (
                <Alert>
                    <Loader2 className="h-4 w-4 animate-spin" />
                    <AlertTitle>{progress.stage}</AlertTitle>
                    <AlertDescription className="space-y-2">
                        <Progress value={progress.percent} className="h-2" />
                        <p className="text-xs">{Math.round(progress.percent)}%</p>
                    </AlertDescription>
                </Alert>
            )}

            {/* Success */}
            {done && (
                <Alert>
                    <CheckCircle className="h-4 w-4 text-green-500" />
                    <AlertTitle>导出成功！</AlertTitle>
                </Alert>
            )}

            {/* Error */}
            {error && (
                <Alert variant="destructive">
                    <AlertTriangle className="h-4 w-4" />
                    <AlertTitle>导出失败</AlertTitle>
                    <AlertDescription>{error}</AlertDescription>
                </Alert>
            )}

            {/* Export button */}
            <Button
                size="lg"
                onClick={handleExport}
                disabled={exporting || missingLines.length > 0 || !outputPath.trim()}
            >
                <Download className="h-4 w-4" />
                {exporting ? '导出中...' : '导出有声书'}
            </Button>
        </div>
    );
}
