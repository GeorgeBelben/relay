type LogoProps = {
    className?: string;
}

export const Logo = ({ className }: LogoProps) => (
    <svg viewBox="0 0 512 512" fill="currentColor" xmlns="http://www.w3.org/2000/svg" className={className}>
        <path d="M256 256H0C0 114.615 114.615 0 256 0V256Z" />
        <path d="M256 512H0C0 370.615 114.615 256 256 256V512Z" />
        <path d="M512 256H256C256 114.615 370.615 0 512 0V256Z" />
    </svg>

)