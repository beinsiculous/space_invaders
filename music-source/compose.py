#!/usr/bin/env python3
"""Author Midnight Toast; explicit regeneration replaces edited project/patch sources."""
import json
from pathlib import Path

SOURCE = Path(__file__).resolve().parent


def save(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')


def note(key, tick, duration=90, velocity=96):
    return dict(pitch=key, start_tick=tick, duration_ticks=duration, velocity=velocity)


def main():
    specifications = [
        ('disco_bass',0,'saw',0.25,0.13,True),
        ('spectral_lead',1,'triangle',0.24,0.20,True),
        ('chord_answer',2,'square',0.12,0.10,True),
        ('midnight_kick',0,'sine',0.80,0.14,False),
        ('midnight_snare',0,'noise',0.40,0.08,False),
        ('disco_hat',0,'noise',0.14,0.023,False),
        ('glass_bones',13,'sine',0.84,0.12,True),
        ('spectral_reed',71,'square',0.75,0.19,True),
        ('halo_bells',14,'triangle',0.90,0.24,True),
        ('toast_thud',32,'saw',0.85,0.10,True),
    ]
    instruments=[]
    for index,(name,program,waveform,gain,decay,pitched) in enumerate(specifications):
        patch=dict(name=name,sample_rate=32000,seed=11700+index,gain=gain,
                   duration_seconds=1.0,pitch_tracking=pitched,
                   oscillators=[dict(waveform=waveform,start_frequency_hz=440.0 if pitched else 175.0,
                                     end_frequency_hz=440.0 if pitched else 42.0,amplitude=1.0)],
                   envelope=dict(attack_seconds=0.001,decay_seconds=decay,sustain_level=0.0,release_seconds=0.05))
        if name in ('disco_bass','toast_thud','spectral_reed','chord_answer'):
            patch['filter']=dict(cutoff_hz={'disco_bass':700.0,'toast_thud':850.0,'spectral_reed':1700.0,'chord_answer':2400.0}[name],
                                 envelope_amount=1.2,resonance=0.18,stages=2)
        if name in ('glass_bones','halo_bells','spectral_lead'):
            patch['oscillators'].append(dict(waveform='sine',start_frequency_hz=1320.0,end_frequency_hz=1320.0,amplitude=0.28))
        if name in ('midnight_snare','disco_hat'):
            patch['filter']=dict(mode='high_pass',cutoff_hz=1300.0 if name=='midnight_snare' else 6500.0,stages=1)
        save(SOURCE/'patches'/f'{name}.json',patch)
        instruments.append(dict(name=name,source=dict(type='synth',patch=f'patches/{name}.json'),midi_program=program,gain=1.0,pan=0.0))
    tracks=[dict(name=name,instrument=instrument,notes=[]) for name,instrument in
            [('Nightwalk',0),('Spectre',1),('Answers',2),('Kick',3),('Snare',4),('Hat',5),
             ('bones',6),('pluck',9),('chime',8),('bonus',8),('hurt',7),('stinger',7),('jam',7)]]
    hooks=[
        [(0,72,110),(240,79,110),(720,75,220),(1200,72,110),(1560,82,240)],
        [(240,79,220),(720,75,110),(960,72,320),(1560,67,240)],
        [(0,75,110),(360,79,220),(960,82,110),(1200,79,110),(1560,75,240)],
        [(0,72,340),(720,67,110),(1080,70,220),(1560,72,360)],
        [(0,84,110),(240,82,110),(720,79,220),(1200,87,110),(1560,84,240)],
        [(240,82,220),(720,79,110),(960,75,320),(1560,79,240)],
        [(0,87,110),(360,84,220),(960,82,110),(1200,79,110),(1560,75,240)],
        [(0,82,340),(720,79,110),(1080,75,220),(1560,72,360)],
    ]
    for bar in range(32):
        start=bar*1920
        reduced=16<=bar<24
        for offset,key in [(0,36),(360,43),(720,48),(960,36),(1320,46),(1680,43)]:
            if not reduced or offset in (0,960):
                tracks[0]['notes'].append(note(key,start+offset,110,110 if offset in (0,960) else 90))
        if not reduced or bar%2==0:
            for offset,key,length in hooks[bar%4+(4 if 8<=bar<16 else 0)]:
                tracks[1]['notes'].append(note(key,start+offset,length,92 if bar>=24 else 80))
        for offset in ([1440] if reduced else [480,1440]):
            for key in ([60,63,67] if bar%2==0 else [60,67,70]):
                tracks[2]['notes'].append(note(key,start+offset,120,78))
        for offset in ([0,960] if reduced else [0,480,960,1440]):
            tracks[3]['notes'].append(note(36,start+offset,90,116))
        for offset in [480,1440]:
            tracks[4]['notes'].append(note(38,start+offset,70,108))
        for offset in range(240,1920,480 if reduced else 240):
            tracks[5]['notes'].append(note(42,start+offset,35,77 if offset%480 else 44))
        if bar%8==7:
            for offset in [1560,1680,1800]:
                tracks[4]['notes'].append(note(38,start+offset,60,85+(offset-1560)//12))
    tracks[0]['notes'].append(note(36,61320,120,112))
    for index,key in enumerate([72,79,75,82]):
        tracks[6]['notes'].append(note(key,index*240,110,118))
    # Continuous short gates let every positional shot find a note, including seams.
    for tick in range(0,61440,120):
        tracks[7]['notes'].append(note([48,55,51,58][(tick//480)%4],tick,120,54))
    tracks[8]['notes'].append(note(48,0,360,94))
    for key in [84,87,91]:
        tracks[9]['notes'].append(note(key,0,360,122))
    for key in [36,43]:
        tracks[10]['notes'].append(note(key,0,360,120))
    for key in [84,87,91,94]:
        tracks[11]['notes'].append(note(key,0,480,124))
    for index,key in enumerate([72,75,79,70]):
        tracks[12]['notes'].append(note(key,index*240,100,78))
    project=dict(format='midi-evil-project',version=2,name='Midnight Toast',tempo_bpm=144,
                 beats_per_bar=4,beat_unit=4,ticks_per_quarter=480,sample_rate=32000,
                 master_gain=0.8,instruments=instruments,tracks=tracks)
    (SOURCE/'midnight_toast.evil').write_text(json.dumps(project,separators=(',',':'))+'\n')
    presets,renders=[],[]
    for index,(name,program,_,_,_,pitched) in enumerate(specifications):
        keys={entry['pitch'] for track in tracks if track['instrument']==index for entry in track['notes']}
        if index in (7,8,9):
            keys.update(entry['pitch'] for entry in tracks[6]['notes'])
        zones=[]
        for key in sorted(keys):
            filename=f'rendered/{name}_{key}.wav'
            renders.append(dict(instrument=index,key=key,ticks=720,wav=filename))
            zones.append(dict(wav=filename,keys=[key,key],root=key,release_seconds=0.05,longest_note_seconds=0.42))
        if pitched:
            presets.append(dict(name=name,bank=0,program=program,zones=zones))
        else:
            percussion=next((preset for preset in presets if preset['bank']==128),None)
            if percussion is None:
                percussion=dict(name='Midnight drums',bank=128,program=0,zones=[])
                presets.append(percussion)
            percussion['zones'].extend(zones)
    save(SOURCE/'bank.json',dict(version=1,name='Midnight Toast',copyright='Original AI synthesis; Be Insiculous; CC0-1.0',presets=presets))
    save(SOURCE/'render.json',dict(seed=11700,sample_rate=32000,velocity=127,gain=0.8,
                                  backing_pitch_classes=[0,3,7,10],cue_pitch_classes=[0,3,7,10],renders=renders))


if __name__=='__main__':
    main()
