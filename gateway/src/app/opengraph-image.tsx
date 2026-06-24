import { ImageResponse } from 'next/og'

export const runtime = 'edge'
export const alt = 'x402 Sovereign Gateway'
export const size = { width: 1200, height: 630 }
export const contentType = 'image/png'

export default async function Image() {
	return new ImageResponse(
		(
			<div
				style={{
					width: '100%',
					height: '100%',
					display: 'flex',
					flexDirection: 'column',
					alignItems: 'center',
					justifyContent: 'center',
					backgroundColor: '#0a0a0a',
					fontFamily: 'monospace',
					backgroundImage: 'radial-gradient(circle at 50% -20%, #1a2f1a 0%, #0a0a0a 60%)',
				}}
			>
				{/* Top thin line */}
				<div style={{ position: 'absolute', top: 0, left: 0, right: 0, height: '4px', backgroundColor: '#00ff41' }} />
				
				{/* Glitchy "Hacker" frame elements */}
				<div style={{ position: 'absolute', top: 40, left: 40, color: '#00ff41', fontSize: 16 }}>[SYS_BOOT]</div>
				<div style={{ position: 'absolute', bottom: 40, right: 40, color: '#444', fontSize: 16 }}>L402 / SOROBAN</div>

				{/* Main Content */}
				<div style={{ display: 'flex', fontSize: 140, color: '#00ff41', fontWeight: 'bold', letterSpacing: '-0.02em', textShadow: '0 0 40px rgba(0, 255, 65, 0.4)' }}>
					x402 MESH
				</div>
				
				<div style={{ display: 'flex', fontSize: 48, color: '#ffffff', marginTop: 40, letterSpacing: '0.4em' }}>
					SOVEREIGN GATEWAY
				</div>
				
				<div style={{ display: 'flex', fontSize: 24, color: '#888888', marginTop: 60, letterSpacing: '0.1em', border: '1px solid #333', padding: '16px 32px', borderRadius: '4px' }}>
					ZERO-TRUST AI ROUTING PROTOCOL
				</div>
			</div>
		),
		{ ...size }
	)
}
