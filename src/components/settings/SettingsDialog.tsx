import { useEffect, useState } from 'react';
import { Save, KeyRound } from 'lucide-react';
import { useSettingsStore } from '../../store/settingsStore';
import { Button } from '../ui/button';
import { Input } from '../ui/input';
import { Label } from '../ui/label';
import { Slider } from '../ui/slider';
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '../ui/select';
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogFooter,
} from '../ui/dialog';

interface SettingsDialogProps {
    onClose: () => void;
}

export default function SettingsDialog({ onClose }: SettingsDialogProps) {
    const settings = useSettingsStore();
    const [apiKey, setApiKey] = useState('');
    const [localEndpoint, setLocalEndpoint] = useState(settings.llmEndpoint);
    const [localModel, setLocalModel] = useState(settings.llmModel);
    const [localTtsModel, setLocalTtsModel] = useState(settings.defaultTtsModel);
    const [localVoice, setLocalVoice] = useState(settings.defaultVoiceName);
    const [localSpeed, setLocalSpeed] = useState(settings.defaultSpeed);
    const [localPitch, setLocalPitch] = useState(settings.defaultPitch);

    useEffect(() => {
        settings.loadSettings();
        settings.loadApiKey('dashscope').then((k) => k && setApiKey(k));
    }, []);

    useEffect(() => {
        setLocalEndpoint(settings.llmEndpoint);
        setLocalModel(settings.llmModel);
        setLocalTtsModel(settings.defaultTtsModel);
        setLocalVoice(settings.defaultVoiceName);
        setLocalSpeed(settings.defaultSpeed);
        setLocalPitch(settings.defaultPitch);
    }, [settings.llmEndpoint, settings.llmModel, settings.defaultTtsModel, settings.defaultVoiceName, settings.defaultSpeed, settings.defaultPitch]);

    const handleSave = async () => {
        settings.set({
            llmEndpoint: localEndpoint,
            llmModel: localModel,
            defaultTtsModel: localTtsModel,
            defaultVoiceName: localVoice,
            defaultSpeed: localSpeed,
            defaultPitch: localPitch,
        });
        await settings.saveSettings();
        if (apiKey) await settings.saveApiKey('dashscope', apiKey);
        onClose();
    };

    return (
        <Dialog open onOpenChange={(open) => !open && onClose()}>
            <DialogContent className="sm:max-w-lg max-h-[90vh] overflow-y-auto">
                <DialogHeader>
                    <DialogTitle>设置</DialogTitle>
                </DialogHeader>

                <section className="space-y-3">
                    <h3 className="text-sm font-semibold flex items-center gap-2">
                        <KeyRound className="h-4 w-4" /> 百炼 API 密钥
                    </h3>
                    <div className="space-y-1.5">
                        <Label>DashScope API Key</Label>
                        <Input
                            type="password"
                            value={apiKey}
                            onChange={(e) => setApiKey(e.target.value)}
                            placeholder="sk-..."
                        />
                    </div>
                    {!apiKey && (
                        <p className="text-xs text-amber-600 dark:text-amber-400">
                            请配置百炼 API 密钥以使用 LLM 剧本生成和 TTS 语音合成功能
                        </p>
                    )}
                </section>

                <section className="space-y-3">
                    <h3 className="text-sm font-semibold">LLM 配置</h3>
                    <div className="space-y-1.5">
                        <Label>API 端点</Label>
                        <Input
                            value={localEndpoint}
                            onChange={(e) => setLocalEndpoint(e.target.value)}
                            placeholder="https://dashscope.aliyuncs.com/compatible-mode/v1"
                        />
                    </div>
                    <div className="space-y-1.5">
                        <Label>模型名称</Label>
                        <Input
                            value={localModel}
                            onChange={(e) => setLocalModel(e.target.value)}
                            placeholder="qwen3.5-plus"
                        />
                    </div>
                </section>

                <section className="space-y-3">
                    <h3 className="text-sm font-semibold">默认 TTS 配置</h3>
                    <div className="grid grid-cols-2 gap-4">
                        <div className="space-y-1.5">
                            <Label>TTS 模型</Label>
                            <Select value={localTtsModel} onValueChange={setLocalTtsModel}>
                                <SelectTrigger className="w-full">
                                    <SelectValue />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="qwen3-tts-flash">Qwen3 TTS Flash</SelectItem>
                                    <SelectItem value="qwen3-tts-instruct-flash">Qwen3 TTS Instruct Flash</SelectItem>
                                    <SelectItem value="cosyvoice-v3-flash">CosyVoice v3 Flash</SelectItem>
                                    <SelectItem value="cosyvoice-v3-plus">CosyVoice v3 Plus</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                        <div className="space-y-1.5">
                            <Label>默认音色</Label>
                            <Input
                                value={localVoice}
                                onChange={(e) => setLocalVoice(e.target.value)}
                                placeholder="Cherry"
                            />
                        </div>
                    </div>
                    <div className="grid grid-cols-2 gap-4">
                        <div className="space-y-2">
                            <Label>默认语速 ({localSpeed.toFixed(1)}x)</Label>
                            <Slider
                                min={0.5}
                                max={2.0}
                                step={0.1}
                                value={[localSpeed]}
                                onValueChange={([v]) => setLocalSpeed(v)}
                            />
                        </div>
                        <div className="space-y-2">
                            <Label>默认音调 ({localPitch.toFixed(1)}x)</Label>
                            <Slider
                                min={0.5}
                                max={2.0}
                                step={0.1}
                                value={[localPitch]}
                                onValueChange={([v]) => setLocalPitch(v)}
                            />
                        </div>
                    </div>
                </section>

                <DialogFooter>
                    <Button variant="outline" onClick={onClose}>取消</Button>
                    <Button onClick={handleSave}>
                        <Save className="h-4 w-4" /> 保存设置
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
