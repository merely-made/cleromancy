import subprocess,time,pathlib
b=pathlib.Path(__file__).parent
p=subprocess.Popen(['python3',str(b/'run-qualified-inspect.py'),'inspect','cold-profile-native','/Users/markik/Code/testing/cleromancy/c4-20261016/profile-native'])
start=time.time();sampled=False
while p.poll() is None:
 if not sampled and time.time()-start>12:
  rows=subprocess.check_output(['ps','-axo','pid,comm'],text=True).splitlines()
  ids=[int(x.split()[0]) for x in rows if x.strip().endswith('/c6-inspect-qualified')]
  if ids:
   subprocess.run(['/usr/bin/sample',str(ids[-1]),'1','-file',str(b/'native-cold-sample.txt')],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
   sampled=True
 time.sleep(.1)
print('exit',p.returncode,'sampled',sampled)
