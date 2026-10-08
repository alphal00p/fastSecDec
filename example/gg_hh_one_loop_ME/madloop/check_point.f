      PROGRAM GGHH_POINT
      IMPLICIT NONE
      INCLUDE 'coupl.inc'
      REAL*8 P(0:3,4),ANS(0:3,0:1),PREC(0:1)
      COMPLEX*16 AMPL(3,10),JAMPL(3,1,1),TOTAL,TRIANGLE,BOXES
      COMPLEX*16 W1(8),W2(8)
      COMMON/ML5_0_AMPL/AMPL
      COMMON/ML5_0_JAMPL/JAMPL
      INTEGER I,J,K,RET

      CALL SETPARA('param_card.dat')
      OPEN(42,FILE='PS.input',STATUS='OLD',ACTION='READ')
      DO I=1,4
        READ(42,*) (P(J,I),J=0,3)
      ENDDO
      CLOSE(42)
      MU_R=300D0
      CALL UPDATE_AS_PARAM(1)
      CALL ML5_0_FORCE_STABILITY_CHECK(.TRUE.)
C     Native HelConfigs.dat row 4 is (1,1,0,0). Helicity filtering is
C     disabled in the parameter card, so the retained flow is this row.
      DO K=1,8
        CALL ML5_0_SLOOPMATRIXHEL_THRES(P,4,ANS,1D-12,PREC,RET)
      ENDDO
C     The sole color basis element is Tr(Ta Tb)=delta_ab/2.
C     Two R2 amplitudes are followed by eight loop diagrams. Native
C     LoopColorFlowCoefs.dat gives +1 for R2 and -1 for each loop.
      TOTAL=JAMPL(1,1,1)/2D0
      TRIANGLE=(AMPL(1,2)-AMPL(1,5)-AMPL(1,6))/2D0
      BOXES=(AMPL(1,1)-AMPL(1,3)-AMPL(1,4)-AMPL(1,7)
     $       -AMPL(1,8)-AMPL(1,9)-AMPL(1,10))/2D0
      WRITE(*,'(A,I0)') 'RET_CODE ',RET
      WRITE(*,'(A,ES26.17E3)') 'RELATIVE_STABILITY ',PREC(0)
      WRITE(*,'(A,2ES26.17E3)') 'A_TOTAL ',TOTAL
      WRITE(*,'(A,2ES26.17E3)') 'A_TRIANGLE ',TRIANGLE
      WRITE(*,'(A,2ES26.17E3)') 'A_BOXES ',BOXES
      WRITE(*,'(A,2ES26.17E3)') 'A_SINGLE_POLE ',JAMPL(2,1,1)/2D0
      WRITE(*,'(A,2ES26.17E3)') 'A_DOUBLE_POLE ',JAMPL(3,1,1)/2D0
C     Fixed-helicity MadLoop divides by 64 initial colors and 2! for
C     the identical Higgs pair. Undo both, with no helicity averaging.
      WRITE(*,'(A,ES26.17E3)') 'MADLOOP_FIXED_HELICITY ',ANS(1,0)
      WRITE(*,'(A,ES26.17E3)') 'COLOR_SUM ',128D0*ANS(1,0)
      WRITE(*,'(A,ES26.17E3)') 'COLOR_SUM_FROM_A ',8D0*ABS(TOTAL)**2
      DO I=1,10
        WRITE(*,'(A,I0,2ES26.17E3)') 'RAW_AMPL ',I,AMPL(1,I)
      ENDDO
      CALL VXXXXX(P(0,1),0D0,1,-1,W1)
      CALL VXXXXX(P(0,2),0D0,1,-1,W2)
      DO I=5,8
        WRITE(*,'(A,I0,2ES26.17E3)') 'EPS1 ',I-5,W1(I)
        WRITE(*,'(A,I0,2ES26.17E3)') 'EPS2 ',I-5,W2(I)
      ENDDO
      END
