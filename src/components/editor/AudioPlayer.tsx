import { useState, useEffect } from 'react';
import { Play, Pause } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Button } from '../ui/button';

interface AudioPlayerProps {
    filePath: string;
}

export default function AudioPlayer({ filePath }: AudioPlayerProps) {
    const [playing, setPlaying] = useState(false);

    useEffect(() => {
        const unlisten = listen('audio-finished', () => {
            setPlaying(false);
        });
        return () => { unlisten.then((fn) => fn()); };
    }, []);

    const toggle = async () => {
        try {
            if (playing) {
                await invoke('stop_audio');
                setPlaying(false);
            } else {
                await invoke('play_audio', { filePath });
                setPlaying(true);
            }
        } catch (e) {
            console.error('Audio playback error:', e);
            setPlaying(false);
        }
    };

    return (
        <Button
            variant="outline"
            size="xs"
            onClick={toggle}
            aria-label={playing ? 'Pause' : 'Play'}
        >
            {playing ? <Pause className="h-3 w-3" /> : <Play className="h-3 w-3" />}
            {playing ? '暂停' : '播放'}
        </Button>
    );
}
