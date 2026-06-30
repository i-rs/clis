import styles from './PulsingDot.module.css'
export default function PulsingDot() {
  return <span className={styles.wrap}><span className={styles.halo} /><span className={styles.dot} /></span>
}
