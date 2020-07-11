@echo off
rustc %~f1
%~dp1%~n1.exe
del %~dp1%~n1.exe
del %~dp1%~n1.pdb